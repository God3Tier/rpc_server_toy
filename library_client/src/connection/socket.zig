const std = @import("std");
const linux = std.os.linux;

pub fn connect(ip: [*:0]const u8, port: u16) !Socket {
    const fd: i32 = @intCast(linux.socket(linux.AF.INET, linux.SOCK.STREAM, 0));
    if (fd < 0) return error.SocketCreateFailed;

    var addr: linux.sockaddr.in = std.mem.zeroes(linux.sockaddr.in);
    addr.family = linux.AF.INET;
    addr.port = std.mem.nativeToBig(u16, port);
    addr.addr = try parseIpv4(ip);

    const rc = linux.connect(fd, @ptrCast(&addr), @sizeOf(linux.sockaddr.in));
    if (@as(isize, @bitCast(rc)) < 0) return error.ConnectFailed;

    return .{ .fd = fd };
}

pub const Socket = struct {
    fd: i32,

    pub fn write_to_stream(self: *Socket, data: []const u8) !void {
        // std.debug.print("Data sent {s}\n", .{data});
        var len_buf: [32]u8 = undefined;

        const len_str = try std.fmt.bufPrint(
            &len_buf,
            "{d} {s}",
            .{data.len, data},
        );

        var len_sent: usize = 0;

        while (len_sent < len_str.len) {
            const rc = linux.write(self.fd, len_str.ptr + len_sent, len_str.len - len_sent);
            const n: isize = @bitCast(rc);
            if (n < 0) return error.WriteFailed;
            len_sent += @intCast(n);
        }
        if (len_sent == 0) return error.ConnectionReset;
    }

    pub fn read_from_stream(self: *Socket, buf: []u8) !usize {
        var got: usize = 0;
        var saw_newline = false;
        while (got < buf.len) {
            const rc = linux.read(self.fd, buf.ptr + got, buf.len - got);
            const n: isize = @bitCast(rc);
            if (n < 0) return error.ReadFailed;
            if (n == 0) break;
            got += @intCast(n);
            if (buf[got - 1] == '\n') {
                saw_newline = true;
                break;
            }
        }

        if (got == 0) return error.ConnectionClosed;
        // std.debug.print("Received {s}\n", .{buf[0..got - 1]}); 
        return if (saw_newline) got - 1 else got;
    }

    pub fn deinit(self: *Socket) void {
        _ = linux.close(self.fd);
        // std.debug.print("Socket deinit fd={}\n", .{self.fd});
    }
};

fn parseIpv4(ip: [*:0]const u8) !u32 {
    const ip_slice = std.mem.span(ip);
    var octets: [4]u8 = undefined;
    var it = std.mem.splitScalar(u8, ip_slice, '.');
    for (&octets) |*o| {
        const tok = it.next() orelse return error.InvalidAddress;
        o.* = try std.fmt.parseInt(u8, tok, 10);
    }
    return @as(u32, octets[0]) | (@as(u32, octets[1]) << 8) |
        (@as(u32, octets[2]) << 16) | (@as(u32, octets[3]) << 24);
}
