import std.os;
import std._str;
import std._vec;

type buf_reader = unsafe obj {
  fn read() -> vec[u8];
};

type buf_writer = unsafe obj {
  fn write(vec[u8] v);
};

fn default_bufsz() -> uint {
  ret 4096u;
}

fn new_buf() -> vec[u8] {
  ret _vec.alloc[u8](default_bufsz());
}

fn new_buf_reader(str path) -> buf_reader {

  unsafe obj fd_buf_reader(int fd, mutable vec[u8] buf) {

    fn read() -> vec[u8] {

      // Ensure our buf is singly-referenced.
      if (_vec.rustrt.refcount[u8](buf) != 1u) {
        buf = new_buf();
      }

      auto len = _vec.len[u8](buf);
      auto vbuf = _vec.buf[u8](buf);
      auto count = os.libc.read(fd, vbuf, len);

      if (count < 0) {
        log "error filling buffer";
        log sys.rustrt.last_os_error();
        fail;
      } else {
        ret buf;
      }
    }

    drop {
      os.libc.close(fd);
    }
  }

  auto fd = os.libc.open(_str.buf(path),
                         os.libc_constants.O_RDONLY() |
                         os.libc_constants.O_BINARY(),
                         0u);

  if (fd < 0) {
    log "error opening file for reading";
    log sys.rustrt.last_os_error();
    fail;
  }
  ret fd_buf_reader(fd, new_buf());
}

type fileflag = tag(append(), create(), truncate());

fn new_buf_writer(str path, vec[fileflag] flags) -> buf_writer {

  unsafe obj fd_buf_writer(int fd) {

    fn write(vec[u8] v) {
      auto len = _vec.len[u8](v);
      auto count = 0u;
      auto vbuf;
      while (count < len) {
        vbuf = _vec.buf_off[u8](v, count);
        auto nout = os.libc.write(fd, vbuf, len);
        if (nout < 0) {
          log "error dumping buffer";
          log sys.rustrt.last_os_error();
          fail;
        }
        count += nout as uint;
      }
    }

    drop {
      os.libc.close(fd);
    }
  }

  let int fflags =
    os.libc_constants.O_WRONLY() |
    os.libc_constants.O_BINARY();

  for (fileflag f in flags) {
    alt (f) {
      case (append())   { fflags |= os.libc_constants.O_APPEND(); }
      case (create())   { fflags |= os.libc_constants.O_CREAT(); }
      case (truncate()) { fflags |= os.libc_constants.O_TRUNC(); }
    }
  }

  auto fd = os.libc.open(_str.buf(path),
                         fflags,
                         os.libc_constants.S_IRUSR() |
                         os.libc_constants.S_IWUSR());

  if (fd < 0) {
    log "error opening file for writing";
    log sys.rustrt.last_os_error();
    fail;
  }
  ret fd_buf_writer(fd);
}

type writer =
  unsafe obj {
    fn write_str(str s);
    fn write_int(int n);
    fn write_uint(uint n);
  };

fn file_writer(str path,
               vec[fileflag] flags)
  -> writer
{
  unsafe obj fw(buf_writer out) {
    fn write_str(str s)   { out.write(_str.bytes(s)); }
    fn write_int(int n)   { out.write(_str.bytes(_int.to_str(n, 10u))); }
    fn write_uint(uint n) { out.write(_str.bytes(_int.uto_str(n, 10u))); }
  }
  ret fw(new_buf_writer(path, flags));
}
