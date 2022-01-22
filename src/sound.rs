use std::io;
use std::io::Read;
use std::process::Child;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;

pub trait AudioStream<E> {
    fn init() -> Result<Self, E> where Self: Sized;
    /// return a single 16bit sample from the audio stream
    fn read_16_sample(&mut self) -> Result<i16, E>;
    /// the default implementation is slow. dont rely on it
    fn read_1_second(&mut self) -> Result<[i16; 16000], E> {
        let mut outbuf = [0; 16000];
        for i in 0..16000 {
            let u16out = self.read_16_sample()?;
            outbuf[i] = u16out;
        }
        Ok(outbuf)
    }
    /// calculate how many bytes are needed to read from the audio stream
    /// in order to read the equivalent of `n` milliseconds. returns a vec
    /// of 16bit samples.
    fn read_n_milliseconds(&mut self, n: f32) -> Result<Vec<i16>, E> {
        let bytes_per_second = 32000.0;
        let num_bytes = (n / 1000.0) * bytes_per_second;
        let num_bytes = num_bytes as usize;
        let mut outbuf: Vec<i16> = Vec::with_capacity(num_bytes);
        for _ in 0..num_bytes {
            let u16out = self.read_16_sample()?;
            outbuf.push(u16out);
        }
        Ok(outbuf)
    }
    fn stop(&mut self) {}
}

// unsafe fn raw_byte_access(s8: &mut [u8]) -> &mut [i16] {
//     slice::from_raw_parts_mut(s8.as_mut_ptr() as *mut i16, s8.len() / 2)
// }

pub struct ParecStream {
    pub child_handle: Child,
    pub stdout: ChildStdout,
    pub is_killed: bool,
}

impl AudioStream<io::Error> for ParecStream {
    fn init() -> Result<Self, io::Error> {
        let mut child = Command::new("parec")
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .arg("--record")
            .arg("--rate=16000")
            .arg("--channels=1")
            .arg("--format=s16be")
            .arg("--latency=10")
            .spawn()?;
        let stdout = child.stdout.take().ok_or(
            io::Error::new(io::ErrorKind::Other, "Failed to spawn parec")
        )?;
        Ok(ParecStream {
            child_handle: child,
            stdout,
            is_killed: false,
        })
    }

    fn read_16_sample(&mut self) -> Result<i16, io::Error> {
        if self.is_killed {
            let e = io::ErrorKind::BrokenPipe;
            return Err(io::Error::new(e, "cannot read sample if parec was killed"));
        }

        let mut buf = [0; 2];
        self.stdout.read_exact(&mut buf)?;
        let u16out = ((buf[0] as i16) << 8) | (buf[1] as i16);
        Ok(u16out)
    }

    fn stop(&mut self) {
        let _ = self.child_handle.kill();
        self.is_killed = true;
    }

    fn read_1_second(&mut self) -> Result<[i16; 16000], io::Error> {
        let mut buf = [0; 32000];
        self.stdout.read_exact(&mut buf)?;
        let mut outbuf = [0; 16000];
        let bufslice = &buf[..];
        let iter = bufslice.chunks(2);
        let mut i = 0;
        for sample in iter {
            let u16out = ((sample[0] as i16) << 8) | (sample[1] as i16);
            outbuf[i] = u16out;
            i += 1;
        }
        Ok(outbuf)
    }

    // couldnt get this to work for some reason
    // fn read_1_second_fast<'a>(&mut self, buf: &'a mut [u8; 32000]) -> Result<&'a mut [i16], io::Error> {
    //     self.stdout.read_exact(buf).unwrap();
    //     println!("Buf[0,1] = {} {}", buf[0], buf[1]);
    //     let i1 = ((buf[0] as i16) << 8) | (buf[1] as i16);
    //     println!("Doing it the slow way: {} ", i1);
    //     let outbuf = unsafe { raw_byte_access(buf) };
    //     println!("The fast way: {}", outbuf[0]);
    //     Ok(outbuf)
    // }
}
