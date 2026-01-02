# Hermes

> [!important]
> This IPC uses Disk I/O to send messages between processes.
> It is not suitable for any real application.

A simple IPC Framework that uses Disk I/O to send data between processes.
The IPC is split into a Client and Server.

To send data `Hermes` uses `XffValues` provided by my `nabu` crate found [here](https://github.com/xqhare/nabu).
To integrate this into your project, I would recommend using my [Athena](https://github.com/xqhare/athena)
crate - it's home to `XffValue` and has all you really need.

## But Why?
Simple: I read the Wikipedia article on [D-Bus](https://en.wikipedia.org/wiki/D-Bus) and had the idea that I could make my own IPC-Bus.

Now, sharing memory regions between processes is a solved problem, but I have no idea how to actually write that myself - and while I brainstormed how to write an IPC myself, I had the brilliant Idea of using files written to disk.
This is the right amount of stupid and scuffed to make me fall in love with the idea and here we are. 

## Usage    

### Client  
```rust
use hermes::Hermes;
use nabu::XffValue;

let mut con = Hermes::new("path/to/pipe").unwrap();
let request: XffValue = "World".into();
if let Err(e) = con.request(request) {
    println!("Error: {:?}", e);
}
let response = con.await_response();
if let Ok(response) = response {
    println!("{:?}", response);
} else {
    println!("Error: {:?}", response.unwrap_err());
}
```

### Server
It is important to note, that this implementation would shut down after the server has handled one request.
```rust
use hermes::Hermes;
use nabu::XffValue;

let mut con = Hermes::new("any/path").unwrap();
let request = con.await_request();
if let Ok(request) = request {
    let response: XffValue = format!("Hello {}!", request.to_string()).into();
    if let Err(e) = con.respond(response) {
        println!("Error: {:?}", e);
    }
} else {
    println!("Error: {:?}", request.unwrap_err());
}
```

#### Alternate Server
This implementation would not shut down after the server has handled one request.
```rust
use hermes::Hermes;
use nabu::XffValue;

let con = Hermes::new("whatever/path").unwrap();
loop {
    let request = con.await_request();
    if let Ok(request) = request {
        let response: XffValue = format!("Hello {:?}!", request).into();
        if let Err(e) = con.respond(response) {
            println!("Error: {:?}", e);
        }
    } else {
        println!("Error: {:?}", request.unwrap_err());
    }
}
```

### Alternate Usage
```rust
use hermes::Hermes;
use nabu::XffValue;

let con = Hermes::new("really/any/path/you/want").unwrap();
let request: XffValue = "World".into();
if let Err(e) = con.request(request) {
    println!("Error: {:?}", e);
}

// Spawn new thread to handle the request
std::thread::spawn(move || {
    let con = Hermes::new("really/any/path/you/want").unwrap();
    let request = con.await_request();
    if let Ok(request) = request {
        let response: XffValue = format!("Hello {:?}!", request).into();
        if let Err(e) = con.respond(response) {
            println!("Error: {:?}", e);
        }
    } else {
        println!("Error: {:?}", request.unwrap_err());
    }
});

let response = con.await_response();
if let Ok(response) = response {
    println!("{:?}", response);
} else {
    println!("Error: {:?}", response.unwrap_err());
}
```
