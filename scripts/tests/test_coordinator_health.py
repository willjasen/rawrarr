"""Read-only HTTP checks for a deployed coordinator health page."""
import argparse
from urllib.request import urlopen
from urllib.error import HTTPError
from html.parser import HTMLParser

class Page(HTMLParser):
    def __init__(self):
        super().__init__()
        self.text = []
    def handle_data(self, data):
        self.text.append(data)
    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        assert tag not in ('iframe', 'object', 'embed'), tag
        assert 'src' not in attrs, 'Health page must be self-contained'
        if 'href' in attrs:
            assert attrs['href'] == '/health', attrs['href']

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', required=True)
    args = parser.parse_args()
    base = args.base_url.rstrip('/')
    bodies = []
    for path in ('/', '/health'):
        with urlopen(base + path, timeout=15) as response:
            assert response.status == 200
            assert response.url == base + path, 'Unexpected redirect'
            assert response.headers.get_content_type() == 'text/html'
            body = response.read().decode()
            page = Page()
            page.feed(body)
            text = ' '.join(page.text)
            for expected in ('Coordinator is responding.', 'Coordinator web service', 'not checked', 'Auto-refreshes every 30 seconds'):
                assert expected in text, expected
            bodies.append(body)
    assert bodies[0] == bodies[1], 'Root must serve the coordinator health page'
    try:
        urlopen(base + '/api/auth/status', timeout=15)
        raise AssertionError('Protected API accepted anonymous request')
    except HTTPError as error:
        assert error.code == 401, error.code
    print('PASS: coordinator root/health HTML, self-contained resources, scope disclosure, protected API')
