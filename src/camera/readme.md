one way to fake the sun is put a cuboid between a DirectionalLight and the camera
where the directional light is pointing towards the camera

tailwind.AMBER_400 looks pretty good for the light
for the cuboid material:

```
    let material = StandardMaterial {
        alpha_mode:            None,
        attenuation_distance:  f32::INFINITY, // this is default but necessary
        base_color:            Color::from(LinearRgba::new(1., 1., 1., 1.)),
        perceptual_roughness:  0.01,
        reflectance:           0.2,
        specular_transmission: 1.,
                               ..default () 
   };       
```

it's probably better if you use a spotlight that you can position and rotate - i just haven't tried it yet
