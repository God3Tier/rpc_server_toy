const std = @import("std");
const Allocator = std.mem.Allocator;

pub fn connect(ipaddr: []const u8, port: i32, allocator: Allocator) *Socket {
    var socket = try allocator(Socket);

    socket.stream = try std.net.tcpConnectToHost(
        std.heap.page_allocator,
        ipaddr,
        port,
    );

    return socket;
}

pub const Socket = struct {
    stream: std.Io.net.Stream,

    pub fn connect(ipaddr: []const u8, port: i32, allocator: Allocator) *Socket {
        var socket = try allocator(Socket);

        socket.stream = try std.net.tcpConnectToHost(
            std.heap.page_allocator,
            ipaddr,
            port,
        );

        return socket;
    }

    pub fn write_to_stream(socket: *Socket, message: []const u8) !void {
        var sent: usize = 0;

        while (sent < message.len()) {
            const n = try socket.stream.write(message[sent..]);
            if (n == 0) {
                return error.ConnectionReset;
            }
            sent += n;
        }
    }

    pub fn read_from_stream(socket: *Socket, buf: []u8) !usize {
        var total: usize = 0;
        while (total < buf.len) {
            const n = try socket.stream.read(buf[total..]);
            if (n == 0) return total; // EOF before buffer filled
            total += n;
        }
        return total;
    }

    pub fn deinit(socket: *Socket) void {
        socket.stream.close();
        socket.deinit();
    }
};
