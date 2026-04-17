use embassy_futures::join::join;
use embassy_time::{Duration, Instant, Timer};

use crate::tasks::{DrivetrainType, EnablesType, ExtensionType, GripperType};

pub async fn handle_game<'d>(
    gripper: &mut GripperType<'d>,
    extension: &mut ExtensionType<'d>,
    drivetrain: &mut DrivetrainType<'d>,
    enables: &mut EnablesType<'d>,
) {
    let start = Instant::now();
    
    Timer::after_secs(10).await;

    enables.0.set_high();
    enables.1.set_high();
    enables.2.set_high();

    join(drivetrain.drive(160.0), gripper.open()).await;
    drivetrain.turn(-90.0).await;
    join(extension.push(), drivetrain.drive(180.0)).await;
    gripper.close().await;
    extension.pull().await;
    drivetrain.drive(40.0).await;
    drivetrain.drive(-210.0).await;

    // Push 1
    drivetrain.turn(90.0).await;
    drivetrain.drive(240.0).await;
    drivetrain.turn(-90.0).await;
    drivetrain.drive(100.0).await;
    drivetrain.turn(90.0).await;
    drivetrain.drive(170.0).await;
    drivetrain.drive(-390.0).await;

    // Leave
    gripper.open().await;
    extension.push().await;
    drivetrain.drive(-90.0).await;
    drivetrain.turn(90.0).await;
    drivetrain.drive(90.0).await;
    drivetrain.turn(-90.0).await;
    drivetrain.drive(480.0).await;
    drivetrain.drive(-50.0).await;
    drivetrain.turn(-70.0).await;
    drivetrain.drive(210.0).await;

    enables.0.set_low();
    enables.1.set_low();
    enables.2.set_low();

    while start.elapsed() < Duration::from_secs(85) {
        Timer::after_millis(100).await
    }

    loop {
        gripper.close().await;
        Timer::after_millis(100).await;
        gripper.open().await;
        Timer::after_millis(100).await;
    }
}
