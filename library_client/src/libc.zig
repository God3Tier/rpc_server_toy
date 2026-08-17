const c = @cImport({
    @cInclude("dlfcn.h");
    @cInclude("sys/types.h");
    @cInclude("sys/stat.h");
});

const OpenFn = *const fn (path: [*:0]const u8, flags: c_int, mode: c_uint) callconv(.c) c_int;
const CloseFn = *const fn (fd: c_int) callconv(.c) c_int;
const ReadFn = *const fn (fd: c_int, buf: [*]u8, count: usize) callconv(.c) isize;
const WriteFn = *const fn (fd: c_int, data: [*]const u8, count: usize) callconv(.c) isize;
const LseekFn = *const fn (fd: c_int, offset: c.off_t, whence: c_int) callconv(.c) c.off_t;
const StatFn = *const fn (ver: c_int, pathname: [*:0]const u8, stat: *c.struct_stat) callconv(.c) c_int;
const UnlinkFn = *const fn (pathname: [*:0]const u8) callconv(.c) c_int;
const GetdirentriesFn = *const fn (fd: c_int, buf: [*]u8, nbytes: c_int, ptr: *c_long) callconv(.c) c_int;

var open_fn: ?OpenFn = null;
var close_fn: ?CloseFn = null;
var read_fn: ?ReadFn = null;
var write_fn: ?WriteFn = null;
var lseek_fn: ?LseekFn = null;
var stat_fn: ?StatFn = null;
var unlink_fn: ?UnlinkFn = null;
var get_direntries_fn: ?GetdirentriesFn = null;

pub fn open(path: [*:0]const u8, flags: c_int, mode: c_uint) c_int {
    if (open_fn == null) {
        const sym = c.dlsym(c.RTLD_NEXT, "open") orelse @panic("dlsym failed for open");
        open_fn = @ptrCast(@alignCast(sym));
    }
    return open_fn.?(path, flags, mode);
}

pub fn close(fd: c_int) c_int {
    if (close_fn == null) {
        const sym = c.dlsym(c.RTLD_NEXT, "close") orelse @panic("dlsym failed for close");
        close_fn = @ptrCast(@alignCast(sym));
    }

    return close_fn.?(fd);
}

pub fn read(fd: c_int, buf: [*]u8, count: usize) isize {
    if (read_fn == null) {
        const sym = c.dlsym(c.RTLD_NEXT, "read") orelse @panic("dlsym failed for read");
        read_fn = @ptrCast(@alignCast(sym));
    }

    return read_fn.?(fd, buf, count);
}

pub fn write(fd: c_int, data: [*]const u8, count: c_int) isize {
    if (write_fn == null) {
        const sym = c.dlsym(c.RTLD_NEXT, "write") orelse @panic("dlysm failed for write");
        write_fn = @ptrCast(@alignCast(sym));
    }

    return write_fn.?(fd, data, count);
}

pub fn lseek(fd: c_int, offset: c.off_t, whence: c_int) c.off_t {
    if (lseek_fn == null) {
        const sym = c.dlsym(c.RTLD_NEXT, "lseek") orelse @panic("dlysm failed for lseek");
        lseek_fn = @ptrCast(@alignCast(sym));
    }

    return lseek_fn.?(fd, offset, whence);
}

pub fn stat(ver: c_int, pathname: [*:0]const u8, stat_struct: *c.struct_stat) c_int {
    if (stat_fn == null) {
        const sym = c.dlsym(c.RTLD_NEXT, "__xstat") orelse @panic("dlysm failed for __xstat");
        stat_fn = @ptrCast(@alignCast(sym));
    }

    return stat_fn.?(ver, pathname, stat_struct);
}

pub fn unlink(pathname: [*:0]const u8) c_int {
    if (unlink_fn == null) {
        const sym = c.dlsym(c.RTLD_NEXT, "unlink") orelse @panic("dlysm failed for unlink");
        unlink_fn = @ptrCast(@alignCast(sym));
    }

    return unlink_fn.?(pathname);
}

pub fn getdirentries(fd: c_int, buf: [*]u8, nbytes: c_int, ptr: *c.off_t) c_int {
    if (get_direntries_fn == null) {
        const sym = c.dlsym(c.RTLD_NEXT, "getdirentries") orelse @panic("dlysm failed for get_direntries");
        get_direntries_fn = @ptrCast(@alignCast(sym));
    }

    return get_direntries_fn.?(fd, buf, nbytes, ptr);
}
