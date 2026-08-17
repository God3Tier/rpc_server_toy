const std = @import("std");
const Allocator = std.mem.Allocator;
const socket = @import("socket.zig");
const constants = @import("../constants.zig");
const dirtree = @import("../dirtree.zig");

pub fn init(ipaddr: []const u8, port: i32, allocator: Allocator) *RPCClient {
    var client = try allocator(RPCClient);
    client.socket = socket.connect(ipaddr, port, allocator);
    client.allocator = allocator;
    return client;
}

pub const RPCClient = struct {
    socket: *socket.Socket,
    allocator: Allocator,

    pub fn open(client: *RPCClient, file_name: []const u8, flag: i32) !i32 {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {s} {d}", .{
            constants.OPEN,
            file_name,
            flag,
        });
        defer client.allocator.free(message);

        socket.*.write_to_stream(message);

        var buf_return: [1024]u8 = undefined;
        try socket.read_from_stream(buf_return[0..]);
        return try std.fmt.ParseInt(buf_return);
    }

    pub fn close(client: *RPCClient, fd: i32) !void {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {d}", .{
            constants.CLOSE,
            fd,
        });
        defer client.allocator.free(message);

        socket.write_to_stream(message);

        var buf_return: [1024]u8 = undefined;
        try socket.read_from_stream(buf_return[0..]);
        return try std.fmt.ParseInt(buf_return);
    }

    pub fn read(client: *RPCClient, fd: i32, count: u32, buf: []u8) !i32 {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {d} {d}", .{
            constants.READ,
            fd,
            count,
        });
        defer client.allocator.free(message);

        socket.write_to_stream(message);
        const n = try socket.read_from_stream(buf[0..]);
        return n;
    }

    pub fn write(client: *RPCClient, fd: i32, data: []const u8) !isize {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {d} {s}", .{
            constants.READ,
            fd,
            data,
        }) catch {
            return -1;
        };
        defer client.allocator.free(message);

        socket.write_to_stream(message);
        var buf_return: [1024]u8 = undefined;
        try socket.read_from_stream(buf_return[0..]);
        return try std.fmt.ParseInt(buf_return);
    }

    pub fn lseek(client: *RPCClient, fd: i32, offset: i64, whence: i32) !i32 {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {d} {d} {d}", .{
            constants.LSEEK,
            fd,
            offset,
            whence,
        });
        defer client.allocator.free(message);

        socket.write_to_stream(message);
        var buf_return: [1024]u8 = undefined;
        try socket.read_from_stream(buf_return[0..]);
        return try std.fmt.ParseInt(buf_return);
    }

    pub fn stat(client: *RPCClient, ver: c_int, path: []const u8, buf: []u8) ![]const u8 {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {d} {s}", .{
            constants.STAT,
            ver,
            path,
        });
        defer client.allocator.free(message);

        socket.write_to_stream(message);
        return try socket.read_from_stream(buf[0..]);
    }

    pub fn unlink(client: *RPCClient, path: []const u8) i32 {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {s}", .{
            constants.UNLINK,
            path,
        });
        defer client.allocator.free(message);

        socket.write_to_stream(message);
        var buf_return: [1024]u8 = undefined;
        try socket.read_from_stream(buf_return[0..]);
        return try std.fmt.ParseInt(buf_return);
    }

    pub fn getdirentries(client: *RPCClient, fd: i32, buf: []u8, nbytes: u32, basep: *i64) !isize {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {d} {d} {d}", .{
            constants.GETDIRENTRIES,
            fd,
            nbytes,
            basep,
        });
        defer client.allocator.free(message);

        socket.write_to_stream(message);
        try socket.read_from_stream(buf[0..]);
        return 0;
    }

    pub fn getDirtree(client: *RPCClient, path: []const u8) *dirtree.DirTree {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {s}", .{
            constants.GETDIRENTRIES,
            path,
        });
        defer client.allocator.free(message);

        socket.write_to_stream(message);
        var buf_return: [1024]u8 = undefined;
        try socket.read_from_stream(buf_return[0..]);

        const allocator = std.heap.ArenaAllocator();
        return dirtree.parse_from_source(buf_return, allocator);
    }
};
