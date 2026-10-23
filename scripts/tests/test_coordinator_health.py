"""Read-only HTTP checks for a deployed coordinator health page."""
import argparse
from urllib.request import urlopen
from urllib.error import HTTPError
from html.parser import HTMLParser

class Page(HTMLParser):
    def __init__(self, health_page=False):
        super().__init__()
        self.text = []
        self.health_page = health_page
    def handle_data(self, data):
        self.text.append(data)
    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if self.health_page:
            assert tag not in ('iframe', 'object', 'embed'), tag
            assert 'src' not in attrs, 'Health page must be self-contained'
        if self.health_page and 'href' in attrs:
            assert attrs['href'] == '/health', attrs['href']

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', required=True)
    args = parser.parse_args()
    base = args.base_url.rstrip('/')
    pages = {}
    for path in ('/', '/health'):
        with urlopen(base + path, timeout=15) as response:
            assert response.status == 200
            assert response.url == base + path, 'Unexpected redirect'
            assert response.headers.get_content_type() == 'text/html'
            body = response.read().decode()
            page = Page(health_page=path == '/health')
            page.feed(body)
            text = ' '.join(page.text)
            pages[path] = (text, body)
    landing_text, _ = pages['/']
    assert 'Sign in' in landing_text, 'Root should serve the signed-out public landing page'
    health_text, _ = pages['/health']
    for expected in ('Roarlay is responding.', 'Roarlay web service', 'not checked', 'Auto-refreshes every 30 seconds'):
        assert expected in health_text, expected
    try:
        urlopen(base + '/api/workspace/nodes', timeout=15)
        raise AssertionError('Protected API accepted anonymous request')
    except HTTPError as error:
        assert error.code == 401, error.code
    print('PASS: coordinator root/health HTML, self-contained resources, scope disclosure, protected API')
