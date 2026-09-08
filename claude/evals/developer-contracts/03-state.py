"""UI state model only; no browser, DOM, focus or accessibility evidence."""
class Screen:
    def __init__(self, rows):
        self.server_rows = rows
        self.query = ""
        self.visible = rows[:]

    def search(self, query):
        self.query = query
        self.visible = [r for r in self.server_rows if query in r]

    def receive(self, rows):
        self.server_rows = rows


if __name__ == "__main__":
    screen = Screen(["apple", "pear"])
    screen.search("app")
    screen.receive(["applet", "plum"])
    assert screen.visible == ["applet"]
    assert screen.query == "app" and screen.server_rows == ["applet", "plum"]
