// TaskForge load test — enqueue jobs, read them back, and hold the system to
// latency + error budgets under a rising VU count.
//
// Run it against the compose stack (see loadtest/README.md):
//   k6 run capstone-taskforge/loadtest/load.js
//   BASE_URL=http://localhost:8080 API_TOKEN=dev-token k6 run load.js
//
// This is deliberately a *closed* enqueue+read loop. The interesting question
// it surfaces isn't "how fast is one POST" — it's what saturates first as VUs
// climb: the API's Postgres pool, the worker's claim throughput, or Postgres
// itself. The README has the "what would you change at 10x" write-up prompts.

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const BASE = __ENV.BASE_URL || 'http://localhost:8080';
const TOKEN = __ENV.API_TOKEN || 'dev-token';

// A custom error rate so a failed *enqueue* (the thing we care about) shows up
// distinctly from k6's built-in http_req_failed, which counts every request.
const enqueueErrors = new Rate('enqueue_errors');

export const options = {
  scenarios: {
    enqueue_and_read: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 20 }, // warm up
        { duration: '1m', target: 20 }, // steady state — read the p95 here
        { duration: '30s', target: 50 }, // push it — where does p95 bend?
        { duration: '30s', target: 0 }, // ramp down
      ],
    },
  },
  thresholds: {
    // The run FAILS (non-zero exit) if any of these are breached. That's the
    // point: a load test with no thresholds is just a graph nobody reads.
    http_req_failed: ['rate<0.01'], // <1% of all requests error
    http_req_duration: ['p(95)<250'], // 95% of requests under 250ms
    enqueue_errors: ['rate<0.01'],
  },
};

const authHeaders = {
  headers: {
    Authorization: `Bearer ${TOKEN}`,
    'Content-Type': 'application/json',
  },
};

export default function () {
  // 1. Enqueue a `send_email` job — the exact job_type the worker binary
  //    registers a handler for, so what we enqueue here actually gets
  //    processed end to end.
  const body = JSON.stringify({
    job_type: 'send_email',
    payload: { to: `user-${__VU}-${__ITER}@example.com`, subject: 'load test' },
  });

  const res = http.post(`${BASE}/jobs`, body, authHeaders);
  const ok = check(res, {
    'enqueue -> 201': (r) => r.status === 201,
    'response carries an id': (r) => r.json('id') !== undefined,
  });
  enqueueErrors.add(!ok);

  // 2. Read it straight back. This GET shares the same Postgres pool the
  //    workers are hammering to claim jobs — read latency climbing under load
  //    is your first sign of pool contention.
  if (ok) {
    const id = res.json('id');
    const got = http.get(`${BASE}/jobs/${id}`, authHeaders);
    check(got, { 'get -> 200': (r) => r.status === 200 });
  }

  sleep(1);
}
