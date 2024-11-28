import asyncio
from SecureStorage.logger import Logger
from server import SecretsServer


class AsyncSecretsServer:
    def __init__(self, host="localhost", port=6000):
        self.host = host
        self.port = port
        self.server = SecretsServer()
        self.logger = Logger().get_logger()

    async def handle_client(self, reader, writer):
        address = writer.get_extra_info("peername")
        self.logger.info(f"New connection from {address}")

        try:
            while True:
                data = await reader.read(1024)
                if not data:
                    break

                msg = eval(data.decode())  # Replace with safer deserialization
                if msg[0] == "get_env":
                    self.logger.info(f"GET: {msg[1]}")
                    responses = []
                    for value in msg[1:]:
                        responses.append(self.server.get_request(value))
                    writer.write(','.join(responses).encode())
                elif msg[0] == "store_env":
                    self.logger.info(f"STORE: {msg[1]}")
                    for value in msg[1:-1]:
                        self.server.store_request(value)
                    writer.write(b"OK")
                else:
                    self.logger.warning(f"Unknown command: {msg[0]}")
                    writer.write(b"ERROR: Unknown command")

                await writer.drain()  # Ensure data is sent to the client
        except Exception as e:
            self.logger.error(f"Error handling client {address}: {e}")
        finally:
            self.logger.info(f"Connection closed: {address}")
            writer.close()
            await writer.wait_closed()

    async def start(self):
        self.logger.info("Starting async server")
        server = await asyncio.start_server(self.handle_client, self.host, self.port)

        async with server:
            self.logger.info(f"Serving on {self.host}:{self.port}")
            await server.serve_forever()


def run_server():
    server = AsyncSecretsServer(host="localhost", port=6000)

    asyncio.run(server.start())


if __name__ == "__main__":
    run_server()
