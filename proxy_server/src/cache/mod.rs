/*
 * This caches all the request incoming from client side into the proxy server
 * It will utilise a doubly linked list to manage the ordering of the program
 * and a hashmap to be able to quickly retrieve the information.
 *
 * The Key will be fd while the entry is the value of the list.
 * Note this needs to be thread safe because of it needing to be read often 
 * and I would also need 
 */
mod entry;

pub struct Cache {}
