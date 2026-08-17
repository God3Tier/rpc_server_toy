const std = @import("std");
const Allocator = std.mem.Allocator;

const DirTreeError = error{ ImproperParsing, TooManyBaseRoot, IncorrectNumSubdirs };

const DirTree = struct {
    name: []const u8,
    num_subdirs: i32,
    children: [*]DirTree,
    allocator: Allocator,

    fn from_sring_basic(dirtree: *DirTree, raw_string: []const u8) void {
        var split = std.mem.split(u8, raw_string, ":");
        dirtree.name = split.next();
        dirtree.num_subdirs = try std.fmt.parseInt(i32, split.next(), 10);
    }

    pub fn parse_from_source(raw_string: []const u8, allocator: Allocator) DirTreeError!*DirTree {
        var base_root = try .allocator(DirTree);

        base_root.allocator = allocator;

        var list = std.ArrayList([]const u8).empty();
        defer list.deinit();

        var iter = std.mem.split(u8, raw_string, "/n");

        var root_done = false;
        while (iter.next()) |string| {
            if (string[0] != '\t') {
                if (root_done) {
                    return DirTreeError.TooManyBaseRoot;
                }
                base_root.from_sring_basic(string);
                root_done = true;
            } else {
                try list.append(allocator, string);
            }
        }

        var sub_dirs = std.ArrayList([]const u8).empty();

        var counter = -1;
        for (list) |dirs| {
            if (dirs[1] != '\t') {
                sub_dirs.append(allocator, dirs[1..] ++ "\n");
                counter += 1;
            } else {
                sub_dirs[counter] ++ dirs[1..];
            }
        }

        if (sub_dirs.len() != base_root.num_subdirs) {
            return DirTreeError.IncorrectNumSubdirs;
        }

        for (0..sub_dirs.len()) |i| {
            (base_root.children + i) = parse_from_source(sub_dirs[i], allocator);
        }

        return base_root;
    }

    // TODO: Write the string function 
    // pub fn to_string(dirtree: *DirTree, indent: u32) ![]const u8 {
        

    // }

    pub fn free(root: *DirTree) !void {
        for (0..root.num_subdirs) |i| {
            (root.children + i).free();
        }
        root.deinit();
    }
};
