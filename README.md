# vtracer-worker

## Plan

We need a worker that accepts an image file sends it to a vtracer converter
and recieves an array of path elements that I can combine and return to
the user on the main thread.

## Worker

Can be built utilizing realness's webpack config for workers

## order

- Get the converter working on the console utilizing a test image.
- Implement a worker version
- Tie into existing worker setup
  - Will need to modify the currenct workflow to have a seperate image resizing
    - This will be done on own worker
    - The other workers will needto wait for this first step to complete
  - Once resized you can then call the 3 workers in parallel
    - (gradients, potrace, vtracer) then (null, svgo, svgo)
  - When finished Potrace and vtracer worker optimizing svg via svgo
  - Look into gziping svg to shrink if files are larger than 600kb
  - Optimize vtracer to output highest quality with the smallest size that will work
  - Will need to convert posters to a folder style file system
    - `/+16282281824/${some-date}/index.html`
      - `color.html`
      - `animations.html`
      - `gradients.html?`
    - This is acceptable performance setup
      - Can load and render small black and white version quickly
      - Stagger loading large color and animation files
      - Will continue to work well on low fidelity networks and efficient hardware
