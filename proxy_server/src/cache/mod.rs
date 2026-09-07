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

pub struct Cache {
    request: HashMap<i32, Node>,
    list: Vec<Node>,
    free_indx: VecDeque<usize>,
    head: Option<usize>,
    tail: Option<usize>,
    next_file_descriptor: i32,
    capacity: u32,
}

impl Cache {
    pub fn new(capacity: u32) -> Cache {
        let mut free_indx = VecDeque::with_capacity(capacity as usize);
        for i in 0..capacity {
            free_indx.push_back(i as usize);
        }
        Cache {
            request: HashMap::with_capacity(capacity as usize),
            list: Vec::with_capacity(capacity as usize),
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
        // SCREW THIS BORROW CHECKER I HAVE TO GO AND TRY SOME NONSENCE
        if let Some(node) = self.get_node(fd) {
            // At this point i gurantee that the node exists, hence I can pull of some shit like this
            let indx = node.indx;

            if let Some(val) = node.value.read_data(read_count) {
                self.touch(indx);
                return Ok(val);
            } else {
                self.touch(indx);
            }
        }

        if let Some(node) = self.get_node_mutable(fd) {
            let message = node.value.fetch_read(read_count).await?;
            return Ok(message);
        }

        // At this point in time I gurantee that it has never existed within my proxy server
        let mut entry = Entry::new(file_name);
        let result = entry.fetch_read(read_count).await?;
        self.insert(entry, fd);
        Ok(result)
    }

    pub fn write_file(&mut self, fd: i32, data: Vec<u8>, append: bool, file_name: String) {
        if let Some(node) = self.get_node_mutable(fd) {
            node.value.write_data(data, append);
        } else {
            let mut entry = Entry::new(file_name);
            entry.write_data(data, append);
            self.insert(entry, fd);
        }
    }

    pub async fn sweep_cache(&mut self) {
    	let mut for_removal = Vec::new(); 
        for (indx, node) in self.list.iter().enumerate() {
            if node.value.is_stale() {
            	for_removal.push(indx); 
            }
        }

        for indx in for_removal {
        	self.remove_node(indx).await; 
        }
    }

    async fn remove_node(&mut self, indx: usize) {
        if self.request.remove(&self.list[indx].fd).is_none() {
            eprintln!("Error. Trying to remove non existant pathway");
            return;
        }
        self.unlink(indx);
        self.free_indx.push_back(indx);

        #[allow(unused)]
        self.list[indx]
            .value
            .flush_dirty_bit()
            .await
            .map_err(|e| eprintln!("Unable to sludh the values {e}"));
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

    fn unlink(&mut self, indx: usize) {
        let (prev, next) = (self.list[indx].prev, self.list[indx].next);

        if let Some(p) = prev {
            self.list[p].next = next;
        } else {
            // If there was no prev, this was the head
            self.head = next;
        }

        if let Some(n) = next {
            self.list[n].prev = prev;
        } else {
            // If there was no next this is the tail
            self.tail = prev;
        }
    }

    fn push_front(&mut self, indx: usize) {
        let (prev, next) = (self.list[indx].prev, self.list[indx].next);

        // Dont need to handle case that it is already head
        if let Some(p) = prev {
            self.list[p].next = next;
        }

        self.head = Some(indx);

        // If there is no next node, dont do anything
        if let Some(n) = next {
            self.list[n].prev = prev;
        }

        if self.tail.is_none() {
            self.tail = Some(indx); // list was empty hence current node becomes tail
        }
    }

    fn get_node_mutable(&mut self, fd: i32) -> Option<&mut Node> {
        if let Some(node) = self.request.get_mut(&fd) {
            return Some(node);
        }
        None
    }
    fn get_node(&self, fd: i32) -> Option<&Node> {
        if let Some(node) = self.request.get(&fd) {
            return Some(node);
        }
        None
    }

    fn insert(&mut self, entry: Entry, fd: i32) {
        match self.free_indx.pop_front() {
            Some(indx) => {
                let node = Node {
                    fd,
                    indx,
                    value: entry,
                    prev: None,
                    next: None,
                };
                self.push_front(indx);
                self.list[indx] = node;
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
                self.list[tail] = node;
                // return tail;
            }
        }
    }
}
