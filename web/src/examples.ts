import type { Format } from './protocol';
export const examples: Record<Format, string> = {
  json: JSON.stringify({
    project: 'twig', description: 'A little clarity for complex data.',
    environments: {
      production: { region: 'ap-south-1', healthy: true, replicas: 3, services: ['api', 'worker', 'gateway'] },
      staging: { region: 'eu-west-1', healthy: true, replicas: 1 }
    },
    team: [{ name: 'Ada', role: 'Engineering', active: true }, { name: 'Sam', role: 'Design', active: true }],
    features: { local_processing: true, uploads: false, supported_formats: ['JSON', 'YAML', 'HAR'] },
    last_deployed: null
  }, null, 2),
  yaml: 'apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: twig-api\nspec:\n  replicas: 3\n  template:\n    spec:\n      containers:\n        - name: api\n          image: twig:latest\n          ports:\n            - containerPort: 8080\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: twig-api\nspec:\n  ports:\n    - port: 80\n      targetPort: 8080\n',
  har: JSON.stringify({ log: { version: '1.2', creator: { name: 'Twig example', version: '1.0' }, entries: [{ startedDateTime: '2026-01-01T12:00:00Z', time: 42, request: { method: 'GET', url: 'https://example.com/api/status', headers: [] }, response: { status: 200, statusText: 'OK', content: { size: 15, mimeType: 'application/json', text: '{"ready":true}' } }, timings: { send: 1, wait: 40, receive: 1 } }] } }, null, 2)
};
