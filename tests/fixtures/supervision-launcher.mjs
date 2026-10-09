// Synthetic public-launcher boundary. No PM3 imports, IPC or daemon access.
const [, , action, service, ...args] = process.argv;
if (service === 'fixture/output') {
  process.stdout.write('x'.repeat(131072));
} else if (service === 'fixture/stderr') {
  process.stderr.write('x'.repeat(131072));
} else if (service === 'fixture/timeout') {
  setInterval(() => {}, 1000);
} else if (service === 'fixture/argv') {
  process.stdout.write(JSON.stringify({ action, service, args,
    home: process.env.PM3_HOME,
    ipc: process.env.PM3_TEST_IPC_PATH,
    run: process.env.PM3_TEST_RUN_DIR,
    pidfile: process.env.PM3_TEST_PIDFILE_PATH,
  }));
} else {
  const intent = args[args.indexOf('--intent') + 1];
  process.stdout.write(JSON.stringify({ name: service, suspended: true, scope: 'login',
    loginId: 'fixture-login', intentId: intent, acknowledgedAt: 'fixture-time', holds: 1 }));
}
