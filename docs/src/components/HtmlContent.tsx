import parse, { domToReact, Element, Text } from 'html-react-parser';
import type { DOMNode, HTMLReactParserOptions } from 'html-react-parser';
import * as Tabs from '@radix-ui/react-tabs';
import { ReactNode, useMemo } from 'react';
import GraphCanvas from './GraphCanvas';
import { CodeBlock } from './docs';

function classes(node: Element): string[] {
  return String(node.attribs?.class ?? '').split(/\s+/).filter(Boolean);
}

function isElement(node: DOMNode): node is Element {
  return (node as { type?: string }).type === 'tag';
}

function textFrom(nodes: DOMNode[]): string {
  return nodes.map((node) => {
    if ((node as { type?: string }).type === 'text') return (node as unknown as Text).data;
    if (isElement(node)) {
      if (node.name === 'br') return '\n';
      return textFrom((node.children ?? []) as DOMNode[]);
    }
    return '';
  }).join('');
}

function findElement(node: Element, predicate: (candidate: Element) => boolean): Element | undefined {
  if (predicate(node)) return node;
  for (const child of (node.children ?? []) as DOMNode[]) {
    if (isElement(child)) {
      const found = findElement(child, predicate);
      if (found) return found;
    }
  }
  return undefined;
}

function childrenOfClass(node: Element, className: string): Element[] {
  return ((node.children ?? []) as DOMNode[]).filter(isElement).filter((child) => classes(child).includes(className));
}

function LegacyTabs({ node, options }: { node: Element; options: HTMLReactParserOptions }) {
  const list = childrenOfClass(node, 'tab-list')[0];
  const buttons = list ? ((list.children ?? []) as DOMNode[]).filter(isElement).filter((child) => child.name === 'button') : [];
  const panels = childrenOfClass(node, 'tab-panel');
  const items = panels.map((panel, index) => ({
    value: panel.attribs?.['aria-labelledby'] || buttons[index]?.attribs?.id || `tab-${index}`,
    label: textFrom((buttons[index]?.children ?? []) as DOMNode[]).trim() || `Tab ${index + 1}`,
    content: domToReact((panel.children ?? []) as DOMNode[], options),
  }));
  if (!items.length) return null;
  return (
    <Tabs.Root className="tabs" defaultValue={items[0].value}>
      <Tabs.List className="tab-list" aria-label="Documentation options">
        {items.map((item) => <Tabs.Trigger key={item.value} value={item.value}>{item.label}</Tabs.Trigger>)}
      </Tabs.List>
      {items.map((item) => <Tabs.Content key={item.value} value={item.value} className="tab-panel active">{item.content}</Tabs.Content>)}
    </Tabs.Root>
  );
}

function legacyCodeBlock(node: Element, copyLabel: string, copiedLabel: string) {
  const pre = findElement(node, (candidate) => candidate.name === 'pre');
  const bar = findElement(node, (candidate) => classes(candidate).includes('code-bar') || classes(candidate).includes('terminal-bar'));
  if (!pre) return undefined;
  return (
    <CodeBlock
      title={bar ? textFrom((bar.children ?? []) as DOMNode[]).trim() : 'code'}
      language={pre.attribs?.['data-lang']}
      code={textFrom((pre.children ?? []) as DOMNode[])}
      copyLabel={copyLabel}
      copiedLabel={copiedLabel}
    />
  );
}

export default function HtmlContent({ html, copyLabel, copiedLabel }: { html: string; copyLabel: string; copiedLabel: string }) {
  return useMemo(() => {
    const options: HTMLReactParserOptions = {
      replace(node) {
        if (!isElement(node as DOMNode)) return undefined;
        const element = node as Element;
        if (element.name === 'canvas' && element.attribs?.id === 'graph-canvas') return <GraphCanvas />;
        const classNames = classes(element);
        if (classNames.includes('code') || classNames.includes('terminal')) return legacyCodeBlock(element, copyLabel, copiedLabel);
        if (classNames.includes('tabs')) return <LegacyTabs node={element} options={options} />;
        return undefined;
      },
    };
    return parse(html, options) as ReactNode;
  }, [html, copyLabel, copiedLabel]);
}
