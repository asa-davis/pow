# usage
0. find a file you care about and add it to /data
1. `cargo run -- encode [your file]`
2. go ahead and open your file with a text editor. you'll notice your old file is gone now
3. write down the new contents or print them out
4. when you want your input file back, download CalTopo
5. go outside to a large open space
6. start tracking yourself on CalTopo and follow the instructions you copied from the file
7. export your map as GeoJSON
8. `cargo run -- decode whatever.geojson [output file with proper extension]`

there is no other way to decode your input files. don't bother looking in the source code.
