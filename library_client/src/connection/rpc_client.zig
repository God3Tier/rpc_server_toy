const std = @import("std");
const Allocator = std.mem.Allocator;
const socket = @import("socket.zig");
const constants = @import("../constants.zig");
const dirtree = @import("../dirtree.zig");

pub fn init(ipaddr: [*:0]const u8, port: u16, allocator: Allocator) !*RPCClient {
    var client = try allocator.create(RPCClient);
    client.socket = try socket.connect(ipaddr, port);
    client.allocator = allocator;
    return client;
}

pub const RPCClient = struct {
    socket: socket.Socket,
    allocator: Allocator,

    pub fn open(client: *RPCClient, file_name: [*:0]const u8, flag: i32) !i32 {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {s} {d}", .{
            constants.OPEN,
            file_name,
            flag,
        });
        defer client.allocator.free(message);

        try client.socket.write_to_stream(message);
        var buf_return: [1024]u8 = undefined;
        const num = try client.socket.read_from_stream(buf_return[0..1024]);
        const res = std.fmt.parseInt(i32, buf_return[0..num], 10) catch |err| {
            std.debug.print("open Error, {} from string {s}", .{ err, buf_return });
            return -1;
        };
        return res;
    }

    pub fn close(client: *RPCClient, fd: i32) !i32 {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {d}", .{
            constants.CLOSE,
            fd,
        });
        defer client.allocator.free(message);

        try client.socket.write_to_stream(message);
        var buf_return: [1024]u8 = undefined;
        const num = try client.socket.read_from_stream(buf_return[0..1024]);
        const res = std.fmt.parseInt(i32, buf_return[0..num], 10) catch |err| {
            std.debug.print("Close Error, {} from string {s}", .{ err, buf_return });
            return -1;
        };
        return res;
    }

    pub fn read(
        client: *RPCClient,
        fd: i32,
        buf: []u8,
        count: usize,
    ) !i32 {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {d} {d}", .{
            constants.READ,
            fd,
            count,
        });
        defer client.allocator.free(message);
        
        try client.socket.write_to_stream(message);
        
        const num = try client.socket.read_from_stream(buf[0..]);
        return @intCast(num);
    }

    pub fn write(client: *RPCClient, fd: i32, data: []const u8, count: usize) !isize {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {d} {s}", .{
            constants.WRITE,
            fd,
            data[0..@intCast(count)],
        });
        defer client.allocator.free(message);

        try client.socket.write_to_stream(message);
        var buf_return: [1024]u8 = undefined;
        const num = try client.socket.read_from_stream(buf_return[0..1024]);
        const res = std.fmt.parseInt(i32, buf_return[0..num], 10) catch |err| {
            std.debug.print("Write Error, {} from string {s}", .{ err, buf_return });
            return -1;
        };
        
        return res;
    }

    pub fn lseek(client: *RPCClient, fd: i32, offset: i64, whence: i32) !i32 {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {d} {d} {d}", .{
            constants.LSEEK,
            fd,
            offset,
            whence,
        });
        defer client.allocator.free(message);

        try client.socket.write_to_stream(message);
        var buf_return: [1024]u8 = undefined;
        const num = try client.socket.read_from_stream(buf_return[0..1024]);
        const res = std.fmt.parseInt(i32, buf_return[0..num], 10) catch |err| {
            std.debug.print("Lseek Error, {} from string {s}", .{ err, buf_return });
            return -1;
        };
        return res;
    }

    pub fn stat(client: *RPCClient, ver: c_int, path: [*:0]const u8, buf: []u8) !i32 {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {d} {s}", .{
            constants.STAT,
            ver,
            path,
        });
        defer client.allocator.free(message);

        try client.socket.write_to_stream(message);
        _ = try client.socket.read_from_stream(buf[0..]);
        return 1;
    }

    pub fn unlink(client: *RPCClient, path: [*:0]const u8) !i32 {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {s}", .{
            constants.UNLINK,
            path,
        });
        defer client.allocator.free(message);

        try client.socket.write_to_stream(message);
        var buf_return: [1024]u8 = undefined;
        const num = try client.socket.read_from_stream(buf_return[0..1024]);
        const res = std.fmt.parseInt(i32, buf_return[0..num], 10) catch |err| {
            std.debug.print("Unlink Error, {} from string {s}", .{ err, buf_return });
            return -1;
        };
        return res;
    }

    pub fn getdirentries(client: *RPCClient, fd: i32, buf: [*]u8, nbytes: i32, basep: *i64) !isize {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {d} {d} {d}", .{
            constants.GETDIRENTRIES,
            fd,
            nbytes,
            basep.*,
        });
        defer client.allocator.free(message);

        try client.socket.write_to_stream(message);
        const num = try client.socket.read_from_stream(buf[0..1024]);
        return @intCast(num);
    }

    pub fn getDirtree(client: *RPCClient, path: [*:0]const u8) *dirtree.DirTree {
        const message = try std.fmt.allocPrint(client.allocator, "{s} {s}", .{
            constants.GETDIRTREE,
            path,
        });
        defer client.allocator.free(message);

        try client.socket.write_to_stream(message);
        var buf_return: [1024]u8 = undefined;
        try client.socket.read_from_stream(buf_return[0..1024]);

        const allocator = std.heap.ArenaAllocator();
        return dirtree.parse_from_source(buf_return, allocator);
    }

    pub fn deinit(client: *RPCClient) void {
        std.debug.print("Client closed\n", .{});
        client.socket.deinit();
        client.allocator.destroy(client);
    }
};
