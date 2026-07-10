# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain each idea precisely, not
whether a test passes.

1. Someone says "a container is basically a really fast, lightweight VM."
   Explain specifically what's wrong with that sentence — name the two
   Linux kernel mechanisms a container actually relies on, and what a VM
   relies on instead, and use that difference to explain *why* a container
   starts in milliseconds while a VM takes seconds to minutes.
2. What do namespaces do, and what do cgroups do? Give one concrete thing
   each one is responsible for isolating or limiting.
3. Because containers share the host kernel instead of getting their own,
   there's a real security tradeoff versus VMs. What is it, and what's one
   concrete mitigation `phase4-backend-advanced/07-deployment-and-operations/01-docker-compose-and-ci`'s
   `Dockerfile` already uses to reduce the blast radius if that boundary
   were ever broken?
4. Explain the image/container distinction using the class/instance
   analogy from this lesson. If you `docker run` the same image three
   times, what do the three containers share, and what does each one have
   that's entirely its own?
5. Explain, in your own words, why `phase4-backend-advanced/07-deployment-and-operations/01-docker-compose-and-ci`'s
   `Dockerfile` copies `Cargo.toml`/`Cargo.lock` and does a dummy build
   *before* copying `src/`, in terms of how Docker's layer cache is keyed.
   What's the general principle behind the ordering, stated without any
   reference to Rust or Cargo specifically?
