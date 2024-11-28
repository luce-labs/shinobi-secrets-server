""" Secure memory block implementation using mmap """

import mmap


class SecureMemoryBlock:
    def __init__(self, size: int):
        """Create a secure memory block"""
        # Ensure size is page-aligned
        self.size = (size + mmap.PAGESIZE - 1) & ~(mmap.PAGESIZE - 1)

        # Create anonymous mapping
        self._memory = mmap.mmap(
            -1,  # File descriptor (-1 for anonymous mapping)
            self.size,
            flags=mmap.MAP_PRIVATE | mmap.MAP_ANONYMOUS,
            prot=mmap.PROT_READ | mmap.PROT_WRITE,
        )

    def write(self, data: bytes) -> None:
        """Write data to secure memory"""
        if len(data) > self.size:
            raise ValueError(f"Data size {len(data)} exceeds block size {self.size}")

        self.clear()  # Clear existing data
        self._memory.seek(0)
        self._memory.write(data)

    def read(self) -> bytes:
        """Read data from secure memory"""
        self._memory.seek(0)
        return self._memory.read(self.size).rstrip(b"\x00")

    def clear(self) -> None:
        """Securely clear memory block"""
        self._memory.seek(0)
        self._memory.write(b"\x00" * self.size)

    def __del__(self):
        """Cleanup"""
        if hasattr(self, "_memory"):
            self.clear()
            self._memory.close()
