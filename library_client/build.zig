const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const mod = b.createModule(.{
        .root_source_file = b.path("src/interpose.zig"),
        .target = target,
        .optimize = optimize,
        .link_libc = true, // you need libc's headers/ABI, and dlopen/dlsym
    });

    const lib = b.addLibrary(.{
        .name = "mylib",
        .linkage = .dynamic, // this is what makes it a .so
        .root_module = mod,
    });

    // Force the output name to mylib.so (not libmylib.so)
    const install = b.addInstallFileWithDir(
        lib.getEmittedBin(),
        .{ .custom = "" }, // installs into zig-out/
        "mylib.so",
    );
    b.getInstallStep().dependOn(&install.step);
}