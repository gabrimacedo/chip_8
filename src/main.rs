fn main() {
    // reserve the 4kb of ram
    let mut memory: Vec<u8> = vec![0; 4096];
    memory[512] = 0x61;
    memory[513] = 0x05;
    memory[514] = 2;

    let counter = &memory[512];

    // fetch instruction from memory 0 and forth
    // read 2 adressses from counter pointer
    // let a = *counter;
    // counter = &memory[513];
    // let b = *counter;
    // let ab: u16 = a << 8;

    // decode instruction

    // execute instruction

    println!("{:x}", *counter);
    let a = *counter as u16;
    let ab = a << 8;
    println!("{:x}", ab);
    let b = 0x05;
    println!("{:x}", b);
    let ab = ab | b;
    println!("{:x}", ab);
}
