import secrets
from .secure_memory_block import SecureMemoryBlock
from typing import Dict, Optional


class SecureStore:
    def __init__(self):
        self._blocks: Dict[str, SecureMemoryBlock] = {}
        self._keys: Dict[str, bytes] = {}

    def store_secret(self, key: str, value: str) -> None:
        # Generate random key for this secret
        encryption_key = secrets.token_bytes(32)

        # Convert and encrypt the value
        value_bytes = self._encrypt(value.encode(), encryption_key)

        # Allocate memory with some padding
        block_size = len(value_bytes) + 32  # Add padding
        block = SecureMemoryBlock(block_size)

        try:
            block.write(value_bytes)
            self._blocks[key] = block
            self._keys[key] = encryption_key
        except Exception as e:
            block.clear()
            raise Exception(f"Failed to store secret: {e}")

    def get_secret(self, key: str) -> Optional[str]:
        if key not in self._blocks:
            return None

        encrypted_data = self._blocks[key].read()
        decrypted = self._decrypt(encrypted_data, self._keys[key])
        return decrypted.decode()

    def _encrypt(self, data: bytes, key: bytes) -> bytes:
        # Simple XOR encryption (for demonstration)
        key_repeated = key * (len(data) // len(key) + 1)
        return bytes(a ^ b for a, b in zip(data, key_repeated))

    def _decrypt(self, data: bytes, key: bytes) -> bytes:
        return self._encrypt(data, key)
