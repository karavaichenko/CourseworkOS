use std::{io::{Read, Write}, net::TcpStream};


struct Client {
    stream1: Option<TcpStream>,
    stream2: Option<TcpStream>,
    server1_address: String,
    server2_address: String,
}


impl Client {
    fn new(server1_address: String, server2_address: String) -> Self {
        let stream1 = TcpStream::connect(server1_address.clone()).ok();
        let stream2 = TcpStream::connect(server2_address.clone()).ok();
        return Self {stream1, stream2, server1_address, server2_address};
    }

    fn disconnect(&mut self, server_num: i32) {
        match server_num {
            1 => {
                self.stream1 = None;
            },
            2 => {
                self.stream2 = None;
            },
            _ => {

            }
        }
    }

    fn connect(&mut self, server_num: i32) -> Result<(), ()> {
        match server_num {
            1 => {
                self.stream1 = TcpStream::connect(self.server1_address.clone()).ok();
                if self.stream1.is_some() {Ok(())} else {Err(())}
            },
            2 => {
                self.stream2 = TcpStream::connect(self.server1_address.clone()).ok();
                if self.stream2.is_some() {Ok(())} else {Err(())}
            },
            _ => {
                return Err(());
            }
        }
    }

    fn send_request(&mut self, server_num: i32, request: &String) -> Option<String> {
        let mut stream: TcpStream;
        match server_num {
            1 => {
                if self.stream1.is_some() {
                    stream = self.stream1.as_mut().unwrap().try_clone().unwrap();
                } else {
                    if self.connect(server_num).is_ok() {
                        return self.send_request(server_num, request);
                    } else {
                        return None;
                    }
            }
            },
            2 => {
                if self.stream2.is_some() {
                        stream = self.stream2.as_mut().unwrap().try_clone().unwrap();
                    } else {
                        if self.connect(server_num).is_ok() {
                            return self.send_request(server_num, request);
                        } else {
                            return None;
                        }
                }
            },
            _ => {
                return None;
            }
        }
        let write_result = stream.write_all(request.as_bytes());
        if write_result.is_err() {
            print!("Сервер разорвал соединение");
        }
    
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer).expect("Не удалось прочитать из сокета");
        let response = String::from_utf8_lossy(&buffer[..bytes_read]);
    
        return Some(response.into_owned());
    }
}


