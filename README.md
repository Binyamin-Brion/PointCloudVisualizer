# PointCloudVisualizer

## Overview
Every point has position within space. If there are many points that are included together to logically form one unit, then they become a point cloud. 
Since a point by itself cannot be seen as it is infinitely small, it can be represented by a geometrical representation such as a cube. 
If this is done for every point, then a visual representation of the cloud can be created.

## Features
* View static point clouds or dynamic point cloud (cloud with constant updates)
* Cluster detection using DBSCAN
* Shadows using a moveable sun
* Side views of the point cloud from the top and right

## Demos

### Dynamic Point Cloud With Information From Airsim

![Alt Text](https://github.com/Binyamin-Brion/PointCloudVisualizer/blob/master/demos/dynamicPointCloudDemo.gif)

### Cluster Detection (with a change of the epsilon parameter)
![Alt Text](https://github.com/Binyamin-Brion/PointCloudVisualizer/blob/master/demos/clusterDetection.gif)

## Sample Release

A static point cloud is provided in the Release section of the repository. This demo was compiled for Linux and Windows.

## Requirements

* System must support OpenGL 4.5
* Operating System: Windows, Linux (tested on Linux Mint. Requires GLFW3 library to be installed)

## User Guide
A user guide in both PDF and docx form is provided in the 'User Guide' folder, which is itself located in the docs folder.

## Input Guide (for all input, see User Guide)

*	WASDQE keys:
    *	If in main scene: Moves the main scene camera
    *	If in top or right side view (click view to enter and exit): Moves selected view camera
  *	Shadow map view: 
    *	Sun mode (view is clicked once): Move the position of the sun
    * Sun look at mode (view is clicked twice): Move where sun is looking at

* ZX keys:
  *	Changes the epsilon value for the DBSCAN algorithm

* VB keys:
  *	Changes the minimum number of points required for a cluster using the DBSCAN algorithm

*	C key:
    *	Runs the DBSCAN clustering algorithm using the provided epsilon and minimum number of points for cluster parameters

## Notes
Implementation of DBSCAN provided by Open3D:

```
@article{
    Zhou2018,
    author    = {Qian-Yi Zhou and Jaesik Park and Vladlen Koltun},
    title     = {{Open3D}: {A} Modern Library for {3D} Data Processing},
    journal   = {arXiv:1801.09847},
    year      = {2018},
}
```
