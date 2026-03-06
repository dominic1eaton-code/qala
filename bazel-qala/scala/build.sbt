ThisBuild / scalaVersion := "2.13.14"
ThisBuild / version := "0.1.0"
ThisBuild / organization := "io.qala"

lazy val common = Seq(
  scalacOptions ++= Seq("-deprecation", "-feature", "-unchecked")
)

lazy val root = (project in file("."))
  .aggregate(workflow, dataPlatform, notifications)
  .settings(
    name := "qala-scala-services",
    publish / skip := true
  )

lazy val workflow = (project in file("workflow-ci-cd-service"))
  .settings(common)
  .settings(
    name := "workflow-ci-cd-service",
    Compile / run / mainClass := Some("com.qala.workflow.Main")
  )

lazy val dataPlatform = (project in file("data-platform-service"))
  .settings(common)
  .settings(
    name := "data-platform-service",
    Compile / run / mainClass := Some("com.qala.data.Main")
  )

lazy val notifications = (project in file("notifications-service"))
  .settings(common)
  .settings(
    name := "notifications-service",
    Compile / run / mainClass := Some("com.qala.notifications.Main")
  )
