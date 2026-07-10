# 06.1 — Docker & containers

No code in this lesson. `phase4-backend-advanced/07-deployment-and-operations/01-docker-compose-and-ci`
already had you write and study a real multi-stage `Dockerfile` and a real
`docker-compose.yml`. This lesson doesn't repeat that walkthrough — it fills
in the conceptual model *underneath* it: what a container actually is at the
OS level, why that makes it different in kind (not just size) from a VM, and
why the layer-caching trick you already read about works the way it does.

## A container is not a lightweight VM

The most common wrong mental model, and worth naming explicitly so you can
retire it: "a container is a small, fast virtual machine." It isn't, and the
difference isn't a matter of degree — it's a completely different isolation
mechanism.

A **virtual machine** virtualizes hardware. A hypervisor (VMware, KVM,
Hyper-V) presents each VM with what looks like its own CPU, RAM, disk, and
network interface, and each VM boots its own complete operating system
kernel on top of that virtual hardware — its own copy of Linux (or Windows),
independently scheduling processes, managing its own memory, running its own
device drivers. That's why a VM takes seconds to minutes to boot: it's doing
everything a real machine does when it powers on, just against virtualized
hardware instead of physical hardware. It's also why a VM's overhead is
substantial — you're running an entire second kernel, with its own memory
footprint, for every VM on the host.

A **container** virtualizes nothing. It's a set of ordinary Linux processes
running directly on the **host kernel** — the same kernel every other
container and every non-containerized process on that machine is using —
made to *look* isolated using two kernel features:

- **Namespaces** give a process (or group of processes) its own view of
  something the kernel manages globally: its own process ID space (PID 1
  inside the container might be PID 47332 on the host), its own filesystem
  mount table (so `/` inside the container is a different directory tree
  than `/` on the host), its own network stack (its own IP address, its own
  view of what "localhost" means), its own hostname. The process isn't
  running on different hardware — it's running on the same kernel, just with
  the kernel selectively lying to it about what else exists.
- **cgroups** (control groups) limit and account for what a process (or
  group) is allowed to consume — how much CPU, how much memory, how much
  disk I/O — and can throttle or kill it if it exceeds that budget. This is
  what stops one noisy container from starving every other container on the
  same host.

Put those two together and you get something that *behaves* like an
isolated machine — its own filesystem, its own process tree, its own
network identity, bounded resource usage — without any of it being real
hardware virtualization. It's process isolation, dressed up to look like a
machine.

This is exactly why containers start in **milliseconds** where VMs take
**seconds to minutes**: starting a container means asking the kernel that's
*already running* to set up a few new namespaces and a cgroup and then
`exec` your binary into them — no second kernel boots, no BIOS, no bootloader,
nothing. And it's why a single host can comfortably run dozens or hundreds of
containers where it might run single-digit VMs: every container shares one
kernel's memory and CPU scheduler instead of each paying the fixed cost of
its own.

The tradeoff, and it's a real one, not a footnote: because containers share
the host kernel, the isolation is weaker than a VM's. A kernel-level
vulnerability can potentially let a process escape its namespace/cgroup
boundary and reach the host or other containers — something a hypervisor
boundary is generally harder to break through. This is *part of* why
`phase4-backend-advanced/07-deployment-and-operations/01-docker-compose-and-ci`'s
`Dockerfile` runs the app as a non-root `appuser` rather than root: it's one
more layer of defense specifically because the container boundary, while
real, is not the same guarantee a VM's hardware-virtualized boundary is.

## Image vs. container: template vs. running instance

These two words get used almost interchangeably in casual conversation, but
they name genuinely different things, and the distinction matters the
moment you're debugging something:

- An **image** is a read-only filesystem snapshot plus metadata (what to
  run, what port to expose, what user to run as) — a *template*. It doesn't
  run; it just exists on disk (or in a registry), the same way a class
  definition doesn't do anything until you instantiate it. `docker build`
  produces an image. `docker push`/`docker pull` move an image around.
- A **container** is a *running instance* of an image — one specific set of
  namespaces and a cgroup, with a thin **writable layer** stacked on top of
  the image's read-only layers. Anything the running process writes (a temp
  file, a log line to a file instead of stdout, a mutation to some file
  under `/app`) goes into that writable layer, not into the image itself.
  `docker run` produces a container.

The relationship is exactly class-and-instance: one image, `docker run`
five times, gives you five independent containers, each with its own
writable layer, each unaware of what the others have written, all sharing
the same underlying read-only image layers on disk (Docker doesn't
duplicate those — copy-on-write means five containers from one image cost
barely more disk than one). Delete a container and its writable layer goes
with it; the image it was built from is untouched and you can `docker run`
a fresh container from it again at any time. This is also *why* containers
are meant to be treated as disposable — see Factor IX (Disposability) in
`04-the-twelve-factor-app` — anything a container writes that actually
needs to survive the container being destroyed has to live somewhere
outside that writable layer (a volume, or — better, per this lesson's
worked example — a database like Postgres that isn't part of the container
at all).

## Layer caching, briefly revisited

`phase4-backend-advanced/07-deployment-and-operations/01-docker-compose-and-ci/README.md`
already walks through this in detail — read that first if you haven't. The
short version, stated at the conceptual level this lesson is aimed at: an
image is a **stack of layers**, one per `Dockerfile` instruction, and Docker
caches each layer keyed on that instruction plus its inputs. That
`Dockerfile`'s `builder` stage deliberately does:

```dockerfile
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release   # (against dummy src/ files)
COPY src ./src
RUN cargo build --release   # (against the real source)
```

instead of one `COPY . .` followed by one `cargo build --release`, precisely
so that a change to `src/` only invalidates the cache for the *second*
`COPY`/`RUN` pair — the expensive "compile every dependency from scratch"
layer stays cached as long as `Cargo.toml`/`Cargo.lock` haven't changed. The
conceptual point worth taking from this lesson specifically: layer caching
isn't a Rust-specific trick, it's a general property of how Docker images
are built — order your `Dockerfile` instructions from *least frequently
changing* to *most frequently changing*, and every rebuild reuses as much
of the previous build as the actual diff allows. The dependency-manifest-
before-source ordering is just that general principle applied to a Cargo
project specifically.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
