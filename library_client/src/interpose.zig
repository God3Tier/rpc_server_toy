const std = @import("std");
const c = @cImport({
    @cInclude("fcntl.h");
    @cInclude("errno.h");
    @cInclude("sys/stat.h");
    @cInclude("sys/types.h");
});
const libc = @import("libc.zig");
const rpc_client = @import("connection/rpc_client.zig");

var client: ?*rpc_client.RPCClient = null;
var fileMap: ?*std.AutoHashMap(i32, i32) = null;
var gpa: std.heap.DebugAllocator(.{}) = .{};

fn get_file_map() *std.AutoHashMap(i32, i32) {
    if (fileMap == null) {
        const map_ptr = gpa.allocator().create(std.AutoHashMap(i32, i32)) catch @panic("alloc failed");
        map_ptr.* = std.AutoHashMap(i32, i32).init(gpa.allocator());
        fileMap = map_ptr;
    }

    return fileMap.?;
}

fn get_client() *rpc_client.RPCClient {
    if (client == null) {
        const ip = std.c.getenv("server15440") orelse "127.0.0.0.1";
        const port = 15440;
        // std.debug.print("Generating client");
        const allocator = gpa.allocator();
        client = rpc_client.init(ip, port, allocator) catch |err| {
            std.debug.print("failed to init RPC client: {}\n", .{err});
            @panic("client init failed");
        };
        // std.debug.print("Successfully spawned cleint  client");
    }

    return client.?;
}

export fn open(path: [*:0]const u8, flags: c_int, ...) callconv(.c) c_int {
    // var args = @cVaStart();
    // defer @cVaEnd(&args);

    // const mode: c.mode_t = if (flags & c.O_CREAT != 0) @cVaArg(&args, c.mode_t) else 0;
    const path_slice = std.mem.span(path);
    var client_ref = get_client();

    const result = client_ref.open(path_slice, flags) catch |err| {
        std.debug.print("client_ref.open failed: {}\n", .{err});
        c.__errno_location().* = c.EIO;
        return -1;
    };

    if (result < 0) {
        c.__errno_location().* = result;
        return -1;
    }

    const local_fd = libc.open("/dev/null", c.O_RDONLY, 0);
    if (local_fd < 0) {
        // Remote file cannot be used, hence we cannot use it for anything
        // and we remove frm server15440

        _ = client_ref.close(result) catch |err| {
            std.debug.print("failed to close server resource: {}\n", .{err});
        };
        c.__errno_location().* = c.ENOMEM;
        return -1;
    }

    get_file_map().put(local_fd, result) catch |err| {
        std.debug.print("failed to push hashmap: {}\n", .{err});
    };
    return local_fd;
}

export fn close(fd: c_int) callconv(.c) c_int {
    const value = get_file_map().get(fd);

    if (value) |fdServer| {
        var client_ref = get_client();
        const result = client_ref.close(fdServer) catch |err| {
            std.debug.print("failed to close server resource: {}\n", .{err});
            c.__errno_location().* = c.EIO;
            return -1;
        };

        if (result < 0) {
            std.debug.print("failed to close server resource: {}\n", .{result});
            c.__errno_location().* = result;
            return -1;
        }
        const local_result = libc.close(fd);
        if (local_result < 0) {
            std.debug.print("Unknown result from lib implementation {}\n", .{local_result});
            c.__errno_location().* = c.ENOMEM;
            return -1;
        }

        _ = get_file_map().remove(fd);
        if (get_file_map().count() == 0) {
            client_ref.deinit();
            client = null;
            if (fileMap != null) {
                fileMap.?.deinit();
                fileMap = null;
            }
        }

        return local_result;
    } else {
        std.debug.print("Unknown not found in map\n", .{});
    }
    return -1;
}

export fn read(fd: c_int, buf: [*]u8, count: usize) callconv(.c) isize {
    const value = get_file_map().get(fd);

    if (value) |fdServer| {
        var client_ref = get_client();
        const result = client_ref.read(fdServer, buf[0..count], count) catch {
            c.__errno_location().* = c.EIO;
            return -1;
        };

        if (result < 0) {
            c.__errno_location().* = c.ENOMEM;
            return -1;
        }

        return result;
    }

    return libc.read(fd, buf, count);
}

export fn write(fd: c_int, data: [*]const u8, count: usize) callconv(.c) isize {
    const value = get_file_map().get(fd);
    if (value) |fdServer| {
        var client_ref = get_client();
        const result = client_ref.write(fdServer, data[0..count], count) catch |err| {
            std.debug.print("Write error from socket {}", .{err});
            c.__errno_location().* = c.EIO;
            return -1;
        };

        if (result < 0) {
            std.debug.print("Result from client error", .{});
            c.__errno_location().* = c.ENOMEM;
            return -1;
        }

        return result;
    }

    return libc.write(fd, data, count);
}

export fn lseek(fd: c_int, offset: c.off_t, whence: c_int) callconv(.c) c.off_t {
    const value = get_file_map().get(fd);
    if (value) |fdServer| {
        var client_ref = get_client();
        const result = client_ref.lseek(fdServer, offset, whence) catch {
            c.__errno_location().* = c.EIO;
            return -1;
        };

        if (result < 0) {
            c.__errno_location().* = c.ENOMEM;
            return -1;
        }

        return result;
    }

    return libc.lseek(fd, offset, whence);
}

export fn __xstat(ver: c_int, pathname: [*:0]const u8, stat_struct: *c.struct_stat) callconv(.c) c_int {
    var client_ref = get_client();
    var buffer: [1024]u8 = undefined;
    const result = client_ref.stat(ver, pathname, buffer[0..1024]) catch {
        c.__errno_location().* = c.EIO;
        return -1;
    };

    if (result < -1) {
        c.__errno_location().* = c.ENOMEM;
        return -1;
    }

    var it = std.mem.splitScalar(u8, buffer[0..1024], ',');
    stat_struct.* = std.mem.zeroes(c.struct_stat);

    const dev = parseField(c.dev_t, &it) catch {
        c.__errno_location().* = c.EIO;
        return -1;
    };
    const ino = parseField(c.ino_t, &it) catch {
        c.__errno_location().* = c.EIO;
        return -1;
    };
    const mode = parseField(c.mode_t, &it) catch {
        c.__errno_location().* = c.EIO;
        return -1;
    };
    const nlink = parseField(c.nlink_t, &it) catch {
        c.__errno_location().* = c.EIO;
        return -1;
    };
    const uid = parseField(c.uid_t, &it) catch {
        c.__errno_location().* = c.EIO;
        return -1;
    };
    const size = parseField(c.off_t, &it) catch {
        c.__errno_location().* = c.EIO;
        return -1;
    };
    const mtime = parseField(c.time_t, &it) catch {
        c.__errno_location().* = c.EIO;
        return -1;
    };

    stat_struct.st_dev = dev;
    stat_struct.st_ino = ino;
    stat_struct.st_mode = mode;
    stat_struct.st_nlink = nlink;
    stat_struct.st_uid = uid;
    stat_struct.st_size = size;
    stat_struct.st_mtim.tv_sec = mtime;

    return 1;
}

export fn unlink(pathname: [*:0]const u8) callconv(.c) c_int {
    var client_ref = get_client();
    const result = client_ref.unlink(pathname) catch {
        c.__errno_location().* = c.EIO;
        return -1;
    };

    if (result == -1) {
        c.__errno_location().* = c.ENOMEM;
        return -1;
    }

    return 0;
}

export fn getdirentries(fd: c_int, buf: [*]u8, nbytes: c_int, basep: *c.off_t) callconv(.c) c_int {
    const value = get_file_map().get(fd);
    if (value) |fdServer| {
        var client_ref = get_client();
        const result = client_ref.getdirentries(fdServer, buf, nbytes, basep) catch {
            c.__errno_location().* = c.EIO;
            return -1;
        };

        if (result == -1) {
            c.__errno_location().* = c.ENOMEM;
            return -1;
        }
        return @intCast(result);
    }

    return libc.getdirentries(fd, buf, nbytes, basep);
}

export fn getdirtree() void {}

fn parseField(comptime T: type, it: *std.mem.SplitIterator(u8, .scalar)) !T {
    const tok = it.next() orelse return error.MalformedResponse;
    return std.fmt.parseInt(T, tok, 10);
}
