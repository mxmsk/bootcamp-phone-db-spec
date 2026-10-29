#!/usr/bin/env python3
"""Check flock(2): a second process gets EWOULDBLOCK; after kill -9 of the owner the lock is free; a child
process that inherited the descriptor (fork without closing) keeps the lock after the parent dies."""
import os, sys, fcntl, subprocess, time, signal, errno, tempfile
d = tempfile.mkdtemp(dir=os.environ.get("TMPDIR", "."))
p = os.path.join(d, "LOCK")
def try_lock(path):
    fd = os.open(path, os.O_RDWR | os.O_CREAT, 0o600)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB); return fd, None
    except OSError as e:
        os.close(fd); return None, errno.errorcode[e.errno]
holder = subprocess.Popen([sys.executable, "-c",
  "import fcntl,os,sys,time;fd=os.open(sys.argv[1],os.O_RDWR|os.O_CREAT);fcntl.flock(fd,fcntl.LOCK_EX);print('ok',flush=True);time.sleep(60)", p],
  stdout=subprocess.PIPE, text=True)
holder.stdout.readline()
print("2nd process while the owner is alive:", try_lock(p)[1])
holder.send_signal(signal.SIGKILL); holder.wait()
fd, err = try_lock(p)
print("after kill -9 of the owner:", "lock acquired" if fd is not None else err)
# inheritance: the parent takes the lock, a fork child without CLOEXEC outlives the parent
fcntl.flock(fd, fcntl.LOCK_UN); os.close(fd)
r, w = os.pipe()
pid = os.fork()
if pid == 0:
    fd = os.open(p, os.O_RDWR); fcntl.flock(fd, fcntl.LOCK_EX); os.write(w, b"x")
    if os.fork() == 0:
        time.sleep(3); os._exit(0)   # the grandchild holds the inherited descriptor
    os._exit(0)                      # the child (owner) dies, the grandchild is alive
os.read(r, 1); os.waitpid(pid, 0); time.sleep(0.3)
print("after the owner died, while a child with the inherited descriptor is alive:", try_lock(p)[1])
time.sleep(3.5)
print("after the child died:", "lock acquired" if try_lock(p)[0] is not None else "busy")
