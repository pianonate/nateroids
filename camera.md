For a 3D space shooter game similar to Asteroids, the camera system should ensure the player has a clear view of their
spaceship and the surrounding space, while also maintaining orientation and avoiding disorientation during rapid
movements and rotations. Here’s a refined approach to achieve this:

Key Considerations:

	1.	Third-Person Perspective: Typically the best choice for such games as it provides a good view of the spaceship and the targets.
	2.	Camera Follow: The camera should smoothly follow the spaceship, maintaining a consistent distance and angle.
	3.	Up Vector Maintenance: To avoid disorientation, the camera should maintain a consistent up vector, usually aligned with the world’s up direction.
	4.	Dynamic Adjustment: The camera can adjust its position and zoom dynamically based on the spaceship’s speed and the action.

Implementation in Bevy:

	1.	Setup the Spaceship and Camera:
	•	Spawn the spaceship entity.
	•	Spawn the camera entity and set its initial position.
	2.	Camera Follow System:
	•	Implement a system to smoothly follow the spaceship.
	•	Maintain a consistent up vector to avoid disorientation.

Here’s a detailed implementation in Bevy:

```
use bevy::prelude::*;

fn main() {
App::build()
.add_plugins(DefaultPlugins)
.add_startup_system(setup.system())
.add_system(camera_follow.system())
.run();
}

struct Spaceship;

fn setup(mut commands: Commands) {
// Spawn the spaceship
commands
.spawn_bundle(PbrBundle {
transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
..Default::default()
})
.insert(Spaceship);

    // Spawn the camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, -10.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..Default::default()
    });
}

fn camera_follow(
spaceship_query: Query<&Transform, With<Spaceship>>,
mut camera_query: Query<&mut Transform, (With<Camera>, Without<Spaceship>)>,
time: Res<Time>,
) {
if let Ok(spaceship_transform) = spaceship_query.single() {
let mut camera_transform = camera_query.single_mut().unwrap();

        // Define a smoothing factor for the camera movement
        let smoothing_factor = 0.1;

        // Set the target camera position behind and above the spaceship
        let offset = Vec3::new(0.0, 5.0, -10.0);
        let target_position = spaceship_transform.translation + offset;

        // Smoothly interpolate the camera's current position towards the target position
        camera_transform.translation = camera_transform.translation.lerp(target_position, smoothing_factor * time.delta_seconds());

        // Compute a target position for the camera to look at
        let target_look_at_position = spaceship_transform.translation;

        // Keep the camera's up vector aligned with the world's up vector (Y-axis)
        let up_vector = Vec3::Y;

        // Make the camera look at the target position with the specified up vector
        camera_transform.look_at(target_look_at_position, up_vector);
    }
}
```

Explanation:

	1.	Setup:
	•	Spawns a spaceship entity with an initial position.
	•	Spawns a camera entity positioned behind and above the spaceship, looking at it.
	2.	Camera Follow System:
	•	Queries the spaceship’s transform to get its current position.
	•	Smoothly interpolates the camera’s position to follow the spaceship using a damping effect (lerp).
	•	Ensures the camera looks at the spaceship while maintaining the world’s up vector.

Additional Enhancements:

	1.	Dynamic Zoom: Adjust the camera’s distance based on the spaceship’s speed to provide a better view during high-speed maneuvers.
	2.	Collision Detection: Implement collision detection to adjust the camera’s position if it intersects with objects in space.
	3.	HUD Integration: Add HUD elements to assist with targeting and orientation.

This setup provides a stable and clear view of the action, enhancing the player’s experience in a 3D space shooter game.
