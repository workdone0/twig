"""Dependency-free static site build. Guides use the repository Markdown source."""
from pathlib import Path
import html
import re
import shutil

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'website/dist'
PAGES={'README.md':('', 'User guide'), 'CONTRIBUTING.md':('contributing','Contributing'), 'docs/MIGRATION.md':('migration','Migration'), 'docs/ARCHITECTURE.md':('architecture','Architecture'), 'docs/EVALUATION.md':('evaluation','Release evaluation'), 'RELEASE_NOTES.md':('release-notes','Release notes')}

def slug(s): return re.sub(r'[^\w\s-]','',s.lower()).strip().replace(' ','-')
def url(target, source):
    if '://' in target or target.startswith('#'): return target
    path, _, anchor=target.partition('#')
    resolved=(source.parent/path).resolve()
    try: rel=str(resolved.relative_to(ROOT))
    except ValueError: return '#'
    if rel in PAGES: dest='/guide/'+PAGES[rel][0]+'/' if PAGES[rel][0] else '/guide/'
    else: dest='https://github.com/workdone0/twig/blob/master/'+rel
    return dest+('#'+anchor if anchor else '')
def inline(s, source):
    tokens=[]
    def keep(v): tokens.append(v);return f'\x00{len(tokens)-1}\x00'
    s=re.sub(r'`([^`]+)`',lambda m:keep('<code>'+html.escape(m[1])+'</code>'),s)
    s=re.sub(r'\[([^\]]+)\]\(([^)]+)\)',lambda m:keep('<a href="'+html.escape(url(m[2],source),quote=True)+'">'+html.escape(m[1])+'</a>'),s)
    s=html.escape(s)
    s=re.sub(r'\*\*(.+?)\*\*',r'<strong>\1</strong>',s)
    # Restore repeatedly because a link's label can contain a protected code token.
    for _ in range(3): s=re.sub(r'\x00(\d+)\x00',lambda m:tokens[int(m[1])],s)
    return s

def markdown(source):
    lines=source.read_text().splitlines(); out=[];i=0
    while i<len(lines):
        line=lines[i]
        if not line.strip() or line.startswith('[Unreleased]:') or re.match(r'^\[[^\]]+\]:',line): i+=1;continue
        if line.startswith('```'):
            language={'bash':'Bash / zsh','powershell':'PowerShell','json':'JSON','yaml':'YAML'}.get(line[3:].strip(),'Text')
            block=[];i+=1
            while i<len(lines) and not lines[i].startswith('```'):block.append(lines[i]);i+=1
            out.append('<div class="code-block"><div class="code-toolbar"><span>'+language+'</span><button type="button" class="copy-code" aria-label="Copy '+language+' example">Copy</button><span class="copy-feedback" role="status"></span></div><pre tabindex="0"><code>'+html.escape('\n'.join(block))+'</code></pre></div>');i+=1;continue
        heading=re.match(r'^(#{1,6}) (.+)',line)
        if heading:
            level=len(heading[1]);text=heading[2];out.append(f'<h{level} id="{slug(text)}">{inline(text,source)}</h{level}>');i+=1;continue
        if line.startswith('|'):
            rows=[]
            while i<len(lines) and lines[i].startswith('|'):
                cells=[c.strip() for c in lines[i].strip('|').split('|')]
                if not all(re.fullmatch(r'[:\s-]+',c) for c in cells): rows.append(cells)
                i+=1
            out.append('<table>')
            for n,row in enumerate(rows):
                tag='th' if n==0 else 'td';out.append('<tr>'+''.join(f'<{tag}>{inline(c,source)}</{tag}>' for c in row)+'</tr>')
            out.append('</table>');continue
        if re.match(r'^(?:- |\d+\. )',line):
            ordered=bool(re.match(r'^\d',line));tag='ol' if ordered else 'ul';out.append('<'+tag+'>')
            while i<len(lines) and re.match(r'^(?:- |\d+\. )',lines[i]):
                text=re.sub(r'^(?:- |\d+\. )','',lines[i]);i+=1
                while i<len(lines) and lines[i].startswith('  ') and not lines[i].lstrip().startswith('```'):text+=' '+lines[i].strip();i+=1
                out.append('<li>'+inline(text,source)+'</li>')
            out.append('</'+tag+'>');continue
        if line.startswith('<'):i+=1;continue
        text=line;i+=1
        while i<len(lines) and lines[i].strip() and not re.match(r'^(#|\||```|- |\d+\. |<)',lines[i]):text+=' '+lines[i].strip();i+=1
        out.append('<p>'+inline(text,source)+'</p>')
    return '\n'.join(out)

def build():
    OUT.mkdir(parents=True,exist_ok=True)
    for name in ['index.html','style.css','app.js','favicon.svg']:shutil.copyfile(ROOT/'website'/name,OUT/name)
    for installer in ['install.sh', 'install.ps1']:
        shutil.copyfile(ROOT/installer,OUT/installer)
    (OUT/'.nojekyll').touch();(OUT/'CNAME').write_text('twig.wtf\n')
    homepage=(ROOT/'website/index.html').read_text()
    header=re.search(r'<header.*?</header>',homepage,re.S)[0]
    footer=re.search(r'<footer.*?</footer>',homepage,re.S)[0]
    nav=''.join(f'<a href="/guide/{route+"/" if route else ""}">{title}</a>' for route,title in PAGES.values())
    for file,(route,title) in PAGES.items():
        destination=OUT/'guide'/route;destination.mkdir(parents=True,exist_ok=True)
        current='/guide/'+(route+'/' if route else '')
        page_nav=nav.replace(f'href="{current}"',f'aria-current="page" href="{current}"')
        document=f'''<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title} — Twig</title><meta name="description" content="Twig {title.lower()}: practical documentation for the Rust terminal data explorer."><link rel="stylesheet" href="/style.css"><script src="/app.js" defer></script><link rel="icon" href="/favicon.svg"><link rel="canonical" href="https://twig.wtf/guide/{route+'/' if route else ''}"></head><body><a class="skip" href="#main">Skip to content</a>{header}<div class="docs-grid wrap"><nav class="docs-sidebar" aria-label="Documentation"><strong>TWIG / DOCUMENTATION</strong>{page_nav}</nav><main id="main" class="prose">{markdown(ROOT/file)}<p class="edit-page"><a href="https://github.com/workdone0/twig/blob/master/{file}">Edit this page on GitHub ↗</a></p></main></div>{footer}</body></html>'''
        (destination/'index.html').write_text(document)
    (OUT/'robots.txt').write_text('User-agent: *\nAllow: /\nSitemap: https://twig.wtf/sitemap.xml\n')
    routes=['/']+['/guide/'+(route+'/' if route else '') for route,_ in PAGES.values()]
    (OUT/'sitemap.xml').write_text('<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">'+''.join('<url><loc>https://twig.wtf'+r+'</loc></url>' for r in routes)+'</urlset>')
    print(f'Built {len(routes)} pages in {OUT}')
if __name__=='__main__':build()
