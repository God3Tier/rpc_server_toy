const std = @import("std");
const c = @cImport({
    @cInclude("fcntl.h");
    @cInclude("errno.h");
});
const libc = @import("libc.zig");
const rpc_client = @import("connection/rpc_client.zig");
const ip = std.posix.getenv("server15440") orelse error.MissingEnvVar;
const port = 15440;

var client: ?rpc_client.RPCClient = null;
var fileMap: ?std.AutoHashMap(i32, i32) = null;

fn get_file_map() std.AutoHashMap(i32, i32) {
    if (fileMap == null) {
        var gpa = std.heap.DebugAllocator(.{}){};
        defer gpa.deinit();

        const allocator = gpa.allocator();
        fileMap = std.AutoHashMap(i32, i32).init(allocator);
    }

    return fileMap;
}

fn get_client() *rpc_client.RPCClient {
    if (client == null) {
        var gpa = std.heap.DebugAllocator(.{}){};
        defer gpa.deinit();

        const allocator = gpa.allocator();
        client = rpc_client.init(ip, port, allocator);
    }

    return client;
}

export fn open(path: [*:0]const u8, flags: c_int, ...) callconv(.c) c_int {
    var args = @cVaStart();
    defer @cVaEnd(&args);

    const mode: c.mode_t = if (flags & c.O_CREAT != 0) @cVaArg(&args, c.mode_t) else 0;
    const path_slice = std.mem.span(path);
    var client_ref = get_client();

    const result = client_ref.open(path_slice, flags, mode) catch {
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
        _ = client_ref.close(result);
        c.__errno_location().* = c.ENOMEM;
        return -1;
    }

    get_file_map().put(local_fd, result);
    return local_fd;
}

export fn close(fd: c_int) callconv(.c) c_int {
    const value = get_file_map().get(fd);

    if (value) |fdServer| {
        var client_ref = get_client();
        const result = client_ref.close(fdServer) catch {
            c.__errno_location().* = c.EIO;
            return -1;
        };

        if (result < 0) {
            c.__errno_location().* = result;
            return -1;
        }
    }

    const local_result = libc.close(fd);
    if (local_result < 0) {
        c.__errno_location().* = c.ENOMEM;
        return -1;
    }

    get_file_map().swapRemove(fd);
    if (get_file_map().count() == 0) {
        client.deinit();
        client = null;
        get_file_map().deinit();
        fileMap = null;
    }

    return local_result;
}

export fn read(fd: c_int, buf: [*]u8, count: usize) callconv(.c) isize {
    const value = get_file_map().get(fd);

    if (value) |fdServer| {
        var client_ref = get_client();
        const result = client_ref.read(fdServer, buf, count) catch {
            c.__errno_location().* = c.EIO;
            return -1;
        };

        if (result < 0) {
            c.__errno_location().* = c.ENOMEM;
            return -1;
        }
    }

    return libc.read(fd, buf, count);
}

export fn write(fd: c_int, data: [*]const u8, count: c_int) callconv(.c) isize {
    const value = get_file_map().get(fd);
    if (value) |fdServer| {
        var client_ref = get_client();
        const result = client_ref.write(fdServer, data, count) catch {
            c.__errno_location().* = c.EIO;
            return -1;
        };

        if (result < 0) {
            c.__errno_location().* = c.ENOMEM;
            return -1;
        }
    }

    return libc.write(fd, data, count);
}

export fn lseek(fd: c_int, offset: c.off_t, whence: c_int) callconv(.c) c.off_t {
    var client_ref = get_client();
    const result = client_ref.lseek(fd, offset, whence) catch {
        c.__errno_location().* = c.EIO;
        return -1;
    };

    if (result < 0) {
        c.__errno_location().* = c.ENOMEM;
        return -1;
    }

    return libc.lseek(fd, offset, whence);
}

export fn __xstat(ver: c_int, pathname: [*:0]const u8, stat_struct: *c.struct_stat) callconv(.c) c_int {
    var client_ref = get_client();
    const result = client_ref.stat(ver, pathname, stat_struct) catch {
        c.__errno_location().* = c.EIO;
        return "Failure";
    };

    if (std.mem.eql(u8, result, "Failure")) {
        c.__errno_location().* = c.ENOMEM;
        return -1;
    }

    var it = std.mem.splitScalar(u8, result, ",");
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
    var client_ref = get_client();

    const result = client_ref.getdirentries(fd, buf, nbytes, basep) catch {
        c.__errno_location().* = c.EIO;
        return -1;
    };

    if (result == -1) {
        c.__errno_location().* = c.ENOMEM;
        return -1;
    }

    return result;
}

export fn getdirtree() void {}

fn parseField(comptime T: type, it: *std.mem.SplitIterator(u8, .scalar)) !T {
    const tok = it.next() orelse return error.MalformedResponse;
    return std.fmt.parseInt(T, tok, 10);
}
