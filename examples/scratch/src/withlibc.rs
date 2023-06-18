use std::{os::fd::RawFd, mem::MaybeUninit};

use libc::{sockaddr_nl, socket, AF_NETLINK, SOCK_RAW, NETLINK_ROUTE, c_int, getpid};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("errorno {0}")]
    Errno(c_int),
}

pub fn main() {
    println!("With libc");
}

// pub unsafe fn connect() -> Result<(), Error> {

//     let nl_sock = {
//         let ret = socket(AF_NETLINK, SOCK_RAW, NETLINK_ROUTE);
//         if ret < 0 {
//             println!("Failed to open nl socket");
//             return Err(Error::Errno(ret));
//         }
//         ret as RawFd
//     };

//     let pid: u32 = {
//         let ret = getpid();
//         if ret < 0 {
//             println!("Failed to get pid");
//             return Err(Error::Errno(ret));
//         }
//         ret as u32
//     };

//     let mut sock_addr: MaybeUninit<sockaddr_nl> = MaybeUninit::uninit();
//     let sock_addr_ptr = sock_addr.as_mut_ptr();
//     sock_addr_ptr.nl_family = AF_NETLINK as u16;
//     sock_addr_ptr.nl_pid = pid;

//         // pub nl_family: ::sa_family_t,
//         // nl_pad: ::c_ushort,
//         // pub nl_pid: u32,
//         // pub nl_groups: u32

//     Ok(())
// }