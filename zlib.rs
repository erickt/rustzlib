extern mod std;

use libc::{c_int, c_ulong};

#[link_name = "z"]
extern mod libz {
    fn compressBound(len: c_ulong) -> c_ulong;
    fn compress2(dest: *mut u8, destlen: &mut c_ulong,
                 src: *u8, srclen: c_ulong,
                 level: c_int) -> c_int;
    fn uncompress(dest: *mut u8, destlen: &mut c_ulong,
                  src: *u8, srclen: c_ulong) -> c_int;
}

pub enum Error {
    StreamError,
    DataError,
    MemError,
    BufError,
    UnknownError(int),
}

pub fn compress(src: &[u8], level: Option<int>) -> Result<~[u8], Error> {
    let level = match level {
        None => -1,
        Some(level) => level,
    };

    let bounds = libz::compressBound(src.len() as c_ulong) as uint;
    let mut dest = vec::with_capacity(bounds);

    let r = do vec::as_imm_buf(src) |psrc, srclen| {
        do vec::as_mut_buf(dest) |pdest, _destlen| {
            let mut destlen = bounds as c_ulong;

            let r = libz::compress2(
                pdest, &mut destlen,
                psrc, srclen as c_ulong,
                level as c_int
            ) as int;

            // Where 0 == Z_OK
            if r == 0 {
                Ok(destlen as uint)
            } else {
                Err(convert_error(r))
            }
        }
    };

    match r {
        Ok(destlen) => {
            unsafe { vec::raw::set_len(&mut dest, destlen); }
            Ok(dest)
        }
        Err(e) => Err(e)
    }
}

pub fn uncompress(src: &[u8], destlen: uint) -> Result<~[u8], Error> {
    let mut dest = vec::with_capacity(destlen);

    let r = do vec::as_imm_buf(src) |psrc, srclen| {
        do vec::as_mut_buf(dest) |pdest, _destlen| {
            let mut destlen = destlen as c_ulong;

            let r = libz::uncompress(
                pdest, &mut destlen,
                psrc, srclen as c_ulong
            ) as int;

            // Where 0 == Z_OK
            if r == 0 {
                Ok(destlen as uint)
            } else {
                Err(convert_error(r))
            }
        }
    };

    match r {
        Ok(destlen) => {
            unsafe { vec::raw::set_len(&mut dest, destlen); }
            Ok(dest)
        }
        Err(e) => Err(e)
    }
}

fn convert_error(r: int) -> Error {
    match r {
        -2 => StreamError,
        -3 => DataError,
        -4 => MemError,
        -5 => BufError,
        _ => UnknownError(r as int)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        let d = ~[0xdeu8, 0xadu8, 0xd0u8, 0x0du8];
        let c = result::unwrap(compress(d, Some(9)));
        let r = result::unwrap(uncompress(c, d.len()));
        assert r == d;
    }
}
