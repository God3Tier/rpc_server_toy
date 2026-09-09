import ctypes
import os

LIB_PATH = "./zig-client/zig-out/mylib.so"

file_path = {}


def load_lib():
    lib = ctypes.CDLL(LIB_PATH, use_errno=True)

    lib.open.restype = ctypes.c_int
    lib.open.argtypes = [ctypes.c_char_p, ctypes.c_int]  # no varargs via ctypes

    lib.close.restype = ctypes.c_int
    lib.close.argtypes = [ctypes.c_int]

    lib.read.restype = ctypes.c_ssize_t
    lib.read.argtypes = [ctypes.c_int, ctypes.c_void_p, ctypes.c_size_t]

    lib.write.restype = ctypes.c_ssize_t
    lib.write.argtypes = [ctypes.c_int, ctypes.c_void_p, ctypes.c_size_t]

    lib.lseek.restype = ctypes.c_int64  # match c.off_t's real width
    lib.lseek.argtypes = [ctypes.c_int, ctypes.c_int64, ctypes.c_int]

    lib.unlink.restype = ctypes.c_int
    lib.unlink.argtypes = [ctypes.c_char_p]

    return lib


def check_open_close(lib, path: bytes):
    print(f"--- open/close: {path!r} ---")
    fd = lib.open(path, os.O_RDONLY | os.O_CREAT)
    print("open ->", fd)
    if fd < 0:
        errno = ctypes.get_errno()
        print("  errno:", errno, os.strerror(errno))
        return
    ret = lib.close(fd)
    print("close ->", ret)


def check_read(lib, path: bytes, count: int = 64):
    print(f"--- read: {path!r} ---")
    fd = lib.open(path, os.O_RDONLY)
    if fd < 0:
        print("open failed, errno:", ctypes.get_errno())
        return
    buf = ctypes.create_string_buffer(count)
    n = lib.read(fd, buf, count)
    print("read ->", n, "bytes:", buf.raw[: max(n, 0)])
    ret = lib.close(fd)
    print("close ->", ret)


def check_write(lib, path: bytes, data: bytes = b"hello from python\n"):
    print(f"--- write: {path!r} ---")
    fd = lib.open(path, os.O_WRONLY | os.O_CREAT)
    file_path[fd] = path
    if fd < 0:
        print("open failed, errno:", ctypes.get_errno())
        return
    print("Sending Write request")
    n = lib.write(fd, data, len(data) + 1)
    print("Received Write request")
    print("write ->", n)
    ret = lib.close(fd)
    print("close ->", ret)


# def check_unlink(lib, path: bytes):
#     print(f"--- unlink: {path!r} ---")
#     ret = lib.unlink(path)
#     print("unlink ->", ret)
#     if ret < 0:
#         print("  errno:", ctypes.get_errno())


if __name__ == "__main__":
    # use_errno=True on CDLL above is what makes ctypes.get_errno() reflect
    # the value your Zig code set via __errno_location(), not some
    # unrelated Python-side errno.
    ctypes.set_errno(0)

    lib = load_lib()

    check_open_close(lib, b"/tmp/testfile.txt")
    # check_read(lib, b"/tmp/testfile.txt")
    for i in range(15):
        # i = 1;
        check_write(
            lib, f"/tmp/scratch{i}.txt".encode(), f"hello from pyhton{i}".encode()
        )
        check_read(lib, f"/tmp/scratch{i}.txt".encode())
    # check_unlink(lib, b"/tmp/scratch.txt")
    print(file_path)
