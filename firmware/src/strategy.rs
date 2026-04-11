use embassy_futures::join::join;

use crate::tasks::{DrivetrainType, ExtensionType, GripperType};

pub async fn handle_game<'d>(
    gripper: &mut GripperType<'d>,
    extension: &mut ExtensionType<'d>,
    drivetrain: &mut DrivetrainType<'d>,
) {
    join(drivetrain.drive(160.0), gripper.open()).await;
    drivetrain.turn(-90.0).await;
    extension.push().await;
    drivetrain.drive(180.0).await;
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
}
