import type { APIRoute, GetStaticPaths } from 'astro';

export const getStaticPaths = (() => {
  const files = import.meta.glob<string>('../../llms/llms*.txt', {
    eager: true,
    query: '?raw',
    import: 'default',
  });

  return Object.entries(files).map(([path, body]) => ({
    params: { filename: path.slice('../../llms/'.length, -'.txt'.length) },
    props: { body },
  }));
}) satisfies GetStaticPaths;

export const GET: APIRoute = ({ props }) =>
  new Response(props.body, { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
