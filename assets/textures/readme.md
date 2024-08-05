Okay - these [cubemap files](https://en.wikipedia.org/wiki/Cube_mapping) are interesting

I found this workflow on reddit after searching for a while to see if there's any tools that will take an image and turn
it into a cubemap - notably - that is compatible with bevy, which handles the images differently.

It's for creating a bevy compatible skybox from a panorama image and not an existing horizontal cross skybox image.

+ Download a panorama image, such
  as [sunflowers_puresky.jpg](https://dl.polyhaven.org/file/ph-assets/HDRIs/extra/Tonemapped%20JPG/sunflowers_puresky.jpg)
+ Upload it to [panorama-to-cubemap](https://jaxry.github.io/panorama-to-cubemap/) and click on each of the generated
  tiles to download them, then put all of them in a folder - note - panorama-to-cubemap will create the file names you
  need in the next step
+ Download the free [ImageMagick](https://imagemagick.org/script/download.php#macosx) cli tool and run this command from
  within the folder with the tiles you downloaded:

```
magick px.png nx.png py.png ny.png pz.png nz.png -gravity center -append cubemap.png
```

where cubemap.png is the output filename you wish

Now you have a vertically stacked cubemap that bevy's skybox component understands!
