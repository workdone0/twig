"""Validate generated internal links, anchors, metadata and installer parity."""
from pathlib import Path
from html.parser import HTMLParser
from urllib.parse import urlsplit, unquote
from build_site import OUT,ROOT
class Page(HTMLParser):
    def __init__(self, text):super().__init__();self.links=[];self.ids=set();self.title=False;self.viewport=False;self.feed(text)
    def handle_starttag(self,tag,attrs):
        a=dict(attrs)
        if 'id' in a:self.ids.add(a['id'])
        if tag=='title':self.title=True
        if a.get('name')=='viewport':self.viewport=True
        if tag in ['a','link','script','img']:
            value=a.get('href',a.get('src',''))
            if value:self.links.append(value)
pages={p:Page(p.read_text()) for p in OUT.rglob('*.html')}
errors=[]
for path,page in pages.items():
    if not page.title or not page.viewport:errors.append(f'Missing metadata: {path}')
    for link in page.links:
        parts=urlsplit(link)
        if parts.scheme or parts.netloc:continue
        dest=OUT/unquote(parts.path).lstrip('/') if parts.path.startswith('/') else path.parent/unquote(parts.path)
        if not parts.path:dest=path
        if dest.is_dir():dest=dest/'index.html'
        if not dest.exists():errors.append(f'{path}: missing {link}')
        elif parts.fragment and dest in pages and unquote(parts.fragment) not in pages[dest].ids:errors.append(f'{path}: missing anchor {link}')
for installer in ['install.sh', 'install.ps1']:
    assert (OUT/installer).read_bytes()==(ROOT/installer).read_bytes()
if errors:raise SystemExit('\n'.join(errors))
print(f'Validated {len(pages)} pages, internal links and installer parity')
