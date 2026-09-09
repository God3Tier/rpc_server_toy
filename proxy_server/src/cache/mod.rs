/*
 * This caches all the request incoming from client side into the proxy server
 * It will utilise a doubly linked list to manage the ordering of the program
 * and a hashmap to be able to quickly retrieve the information.
 *
 * The Key will be fd while the entry is the value of the list.
 * Note this needs to be thread safe because of it needing to be read often
 * and I would also need to lock any attempt to write. HMMMM FFS
 */
mod entry;

use self::entry::Entry;
use crate::Error;
use std::collections::{HashMap, VecDeque};

struct Node {
    fd: i32,
    indx: usize,
    value: Entry,
    prev: Option<usize>,
    next: Option<usize>,
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.value)
    }
}

impl Default for Node {
    fn default() -> Node {
        Node {
            fd: 0,
            indx: 0,
            value: Entry::new(String::from("DOESN'T EXIST")),
            prev: None,
            next: None,
        }
    }
}

pub struct Cache {
    request: HashMap<i32, usize>,
    list: Vec<Node>,
    free_indx: VecDeque<usize>,
    head: Option<usize>,
    tail: Option<usize>,
    next_file_descriptor: i32,
    #[allow(unused)]
    capacity: u32,
}

impl Cache {
    pub fn new(capacity: u32) -> Cache {
        let mut free_indx = VecDeque::with_capacity(capacity as usize);
        for i in 0..capacity {
            free_indx.push_back(i as usize);
        }

        let mut list = Vec::with_capacity(capacity as usize);
        for _ in 0..capacity {
            list.push(Node::default())
        }
        Cache {
            request: HashMap::with_capacity(capacity as usize),
            list,
            head: None,
            tail: None,
            free_indx,
            next_file_descriptor: 1,
            capacity,
        }
    }

    pub async fn read_file(
        &mut self,
        fd: i32,
        read_count: u32,
        file_name: String,
    ) -> Result<Vec<u8>, Error> {
        // println!("State of cache: {:?}", self.list);
        if let Some(indx) = self.get_node(fd) {
            let node_read = unsafe { self.list.get_unchecked(indx) };
            let indx = node_read.indx;
            let result = node_read.value.read_data(read_count);

            match result {
                Some(val) => {
                    self.touch(indx);
                    return Ok(val);
                }
                None => {
                    self.touch(indx);
                    let node_write = unsafe { self.list.get_unchecked_mut(indx) };
                    let message = node_write.value.fetch_read(read_count).await?;
                    return Ok(message);
                }
            }
        }

        // println!("State of cache: {:?}", self.list);

        // At this point in time I gurantee that it has never existed within my proxy server
        // println!("Not found in cache\n State of cache: {:?}", self.list);
        let mut entry = Entry::new(file_name);
        let result = entry.fetch_read(read_count).await?;
        self.insert(entry, fd).await;
        Ok(result)
    }

    pub async fn write_file(&mut self, fd: i32, data: Vec<u8>, append: bool, file_name: String) {
        if let Some(indx) = self.get_node(fd) {
            let node = unsafe { self.list.get_unchecked_mut(indx) };
            node.value.write_data(data, append);
        } else {
            let mut entry = Entry::new(file_name);
            entry.write_data(data, append);
            self.insert(entry, fd).await;
            // println!("State of cache: {:?}", self.list);
        }
    }

    pub async fn sweep_cache(&mut self) -> Vec<i32> {
        let mut for_removal = Vec::new();
        let mut fd_removed = Vec::new();
        for (indx, node) in self.list.iter().enumerate() {
            if node.value.is_stale() {
                for_removal.push(indx);
                fd_removed.push(node.fd);
            }
        }

        for indx in &for_removal {
            self.remove_node(*indx).await;
        }

        fd_removed
    }

    async fn remove_node(&mut self, indx: usize) {
        if self
            .request
            .remove(&unsafe { self.list.get_unchecked(indx) }.fd)
            .is_none()
        {
            eprintln!("Error. Trying to remove non existant pathway");
            return;
        }

        self.unlink(indx);
        self.free_indx.push_back(indx);

        println!("Removing node {:?}", self.list[indx]);
        #[allow(unused)]
        unsafe { self.list.get_unchecked_mut(indx) }
            .value
            .flush_dirty_bit()
            .await
            .map_err(|e| eprintln!("Unable to flush the values {e}"));
    }

    pub fn generate_fd(&mut self) -> i32 {
        let res = self.next_file_descriptor % i32::MAX;
        self.next_file_descriptor += 1;
        res
    }

    fn touch(&mut self, indx: usize) {
        self.unlink(indx);
        self.push_front(indx);
    }

    fn unlink(&mut self, idx: usize) {
        let (prev, next) = (
            unsafe { self.list.get_unchecked(idx) }.prev,
            unsafe { self.list.get_unchecked(idx) }.next,
        );

        match prev {
            Some(p) => unsafe { self.list.get_unchecked_mut(p) }.next = next,
            None => self.head = next, // idx was head
        }
        match next {
            Some(n) => unsafe { self.list.get_unchecked_mut(n) }.prev = prev,
            None => self.tail = prev, // idx was tail
        }
    }

    fn push_front(&mut self, idx: usize) {
        let old_head = self.head;

        let node = unsafe { self.list.get_unchecked_mut(idx) };
        node.prev = None;
        node.next = old_head;

        if let Some(h) = old_head {
            unsafe { self.list.get_unchecked_mut(h) }.prev = Some(idx);
        }

        self.head = Some(idx);

        if self.tail.is_none() {
            self.tail = Some(idx);
        }
    }

    fn get_node(&self, fd: i32) -> Option<usize> {
        if let Some(node) = self.request.get(&fd) {
            return Some(*node);
        }
        None
    }

    async fn insert(&mut self, entry: Entry, fd: i32) {
        match self.free_indx.pop_front() {
            Some(indx) => {
                println!("Inserting {:?} at indx : {}", entry, indx);
                let node = Node {
                    fd,
                    indx,
                    value: entry,
                    prev: None,
                    next: None,
                };
                unsafe {
                    let pointer = self.list.get_unchecked_mut(indx);
                    *pointer = node;
                };

                self.push_front(indx);
                self.request.insert(fd, indx);
                // return indx;
            }
            None => {
                let tail = self
                    .tail
                    .expect("If tail is not set, implies that caopacity was 0");

                let node = Node {
                    fd,
                    indx: tail,
                    value: entry,
                    prev: None,
                    next: None,
                };

                self.unlink(tail);
                self.push_front(tail);
                unsafe {
                    let pointer = self.list.get_unchecked_mut(tail);
                    *pointer = node;
                };
                self.request.insert(fd, tail);
                // return tail;
            }
        }
    }
}
