use embassy_futures::join::join;
use embassy_time::{Duration, Instant, Timer};

use crate::tasks::{DrivetrainType, EnablesType, GripperType};

pub async fn handle_game<'d>(
    gripper: &mut GripperType<'d>,
    drivetrain: &mut DrivetrainType<'d>,
    enables: &mut EnablesType<'d>,
) {
    let start = Instant::now();

    enables.0.set_high();
    enables.1.set_high();

    join(drivetrain.drive(160.0), gripper.open()).await;
    drivetrain.turn(-90.0).await;
    drivetrain.drive(180.0).await;
    gripper.close().await;
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

    while start.elapsed() < Duration::from_secs(85) {
        Timer::after_millis(100).await
    }

    while start.elapsed() < Duration::from_secs(100) {
        gripper.close().await;
        Timer::after_millis(100).await;
        gripper.open().await;
        Timer::after_millis(100).await;
    }
}
