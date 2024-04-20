# Composing a Peak Graph

In this guide, you'll learn how to compose this peak graph:

![](./peak_graph.png)

It will have a grid backdrop and a unit ruler. Also, it will be scaled by
decibels and show all peak information of the last 10 seconds that is between
-32dB and 8dB.

In order to layer the graph with a grid backdrop and a unit ruler, we'll use
VIZIA's `ZStack`. And we will store the peak information inside a `PeakBuffer` -
this is one of Cyma's own buffers.

Our starting point is a barebones nih-plug + VIZIA Plugin. It's assumed that you
already know how to use both of these frameworks.
