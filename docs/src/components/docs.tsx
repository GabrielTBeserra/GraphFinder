import { ReactNode, useMemo, useState } from 'react';
import { Check, Copy, Info, Lightbulb, TriangleAlert } from 'lucide-react';
import clsx from 'clsx';

export function DocSection({ id, index, title, children }: { id: string; index: string; title: string; children: ReactNode }) {
  return (
    <section id={id}>
      <h2><span className="idx">{index}</span>{title}<a className="anchor" href={`#${id}`} aria-label={`Link to ${title}`}>#</a></h2>
      {children}
    </section>
  );
}

export function Callout({ type, children }: { type: 'info' | 'tip' | 'warn'; children: ReactNode }) {
  const Icon = type === 'tip' ? Lightbulb : type === 'warn' ? TriangleAlert : Info;
  return <div className={`callout ${type}`}><Icon aria-hidden="true" /><p>{children}</p></div>;
}

function tokeniseJsonLine(line: string, keyPrefix: string): ReactNode[] {
  const parts: ReactNode[] = [];
  const pattern = /("[^"]*")(\s*:)?|\b(true|false|null|\d+(?:\.\d+)?)\b/g;
  let last = 0;
  let match: RegExpExecArray | null;
  let index = 0;
  while ((match = pattern.exec(line))) {
    if (match.index > last) parts.push(line.slice(last, match.index));
    if (match[1]) parts.push(<span className="k" key={`${keyPrefix}-${index++}`}>{match[1]}</span>, match[2] ?? '');
    else if (match[3]) parts.push(': ', <span className="s" key={`${keyPrefix}-${index++}`}>{match[3]}</span>);
    else if (match[4]) parts.push(<span className="n" key={`${keyPrefix}-${index++}`}>{match[4]}</span>);
    last = pattern.lastIndex;
  }
  if (last < line.length) parts.push(line.slice(last));
  return parts;
}

function HighlightedCode({ code, language }: { code: string; language?: string }) {
  const lines = useMemo(() => code.replace(/\n$/, '').split('\n'), [code]);
  return (
    <>
      {lines.map((line, lineIndex) => {
        let content: ReactNode = line;
        if ((language === 'bash' || language === 'powershell') && /^\s*#/.test(line)) {
          content = <span className="c">{line}</span>;
        } else if (language === 'bash' || language === 'powershell') {
          const match = line.match(/^(\s*)(\$ )?(\S+)(.*)$/);
          if (match) {
            content = <>{match[1]}{match[2] ? <span className="p">$ </span> : null}<span className="k">{match[3]}</span>{match[4]}</>;
          }
        } else if (language === 'json') {
          content = tokeniseJsonLine(line, `l${lineIndex}`);
        } else if (language === 'markdown') {
          if (/^#{1,6}\s/.test(line)) content = <span className="k">{line}</span>;
          else {
            const chunks = line.split(/(\[\[[^\]]+\]\]|(?<!\S)#[a-z][\w-]*)/g);
            content = chunks.map((chunk, chunkIndex) => {
              if (/^\[\[/.test(chunk)) return <span className="o" key={chunkIndex}>{chunk}</span>;
              if (/^#[a-z]/.test(chunk)) return <span className="n" key={chunkIndex}>{chunk}</span>;
              return <span key={chunkIndex}>{chunk}</span>;
            });
          }
        }
        return <span key={lineIndex}>{content}{lineIndex < lines.length - 1 ? '\n' : ''}</span>;
      })}
    </>
  );
}

export function CodeBlock({ title, language, code, copyLabel = 'Copy', copiedLabel = 'Copied' }: {
  title: string;
  language?: string;
  code: string;
  copyLabel?: string;
  copiedLabel?: string;
}) {
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    const source = code.replace(/^\$ /gm, '').trim();
    try {
      await navigator.clipboard.writeText(source);
    } catch {
      const textarea = document.createElement('textarea');
      textarea.value = source;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand('copy');
      textarea.remove();
    }
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1600);
  };
  return (
    <div className={language === 'bash' || title === 'terminal' ? 'terminal' : 'code'}>
      <div className={language === 'bash' || title === 'terminal' ? 'terminal-bar' : 'code-bar'}>
        {title === 'terminal' && <span className="dots"><i /><i /><i /></span>}
        {title}
        <button className={clsx('copy-btn', copied && 'done')} type="button" onClick={copy} aria-label={copyLabel}>
          {copied ? <Check aria-hidden="true" /> : <Copy aria-hidden="true" />}<span>{copied ? copiedLabel : copyLabel}</span>
        </button>
      </div>
      <pre data-lang={language}><code><HighlightedCode code={code} language={language} /></code></pre>
    </div>
  );
}

export function DataTable({ headers, rows }: { headers: ReactNode[]; rows: ReactNode[][] }) {
  return (
    <div className="table-wrap">
      <table>
        <thead><tr>{headers.map((header, index) => <th key={index}>{header}</th>)}</tr></thead>
        <tbody>{rows.map((row, rowIndex) => <tr key={rowIndex}>{row.map((cell, cellIndex) => <td key={cellIndex}>{cell}</td>)}</tr>)}</tbody>
      </table>
    </div>
  );
}
