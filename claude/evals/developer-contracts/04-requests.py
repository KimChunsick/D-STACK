"""Real asyncio tasks; a UI controller specimen, not a browser integration test."""
import asyncio


class Controller:
    def __init__(self):
        self.value = None
        self.submissions = 0

    async def load(self, value, ready):
        await ready.wait()
        self.value = value

    async def submit(self, transport):
        self.submissions += 1
        await transport.wait()


async def check():
    c = Controller()
    old_ready, new_ready = asyncio.Event(), asyncio.Event()
    old = asyncio.create_task(c.load("old", old_ready))
    new = asyncio.create_task(c.load("new", new_ready))
    await asyncio.sleep(0)
    new_ready.set()
    await new
    old_ready.set()
    await old
    assert c.value == "new", "late completion replaced current request"
    sent = asyncio.Event()
    a = asyncio.create_task(c.submit(sent))
    b = asyncio.create_task(c.submit(sent))
    await asyncio.sleep(0)
    sent.set()
    await asyncio.gather(a, b)
    assert c.submissions == 1, "concurrent submissions duplicated the effect"


if __name__ == "__main__":
    asyncio.run(check())
