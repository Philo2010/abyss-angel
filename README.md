# abyss-angel

Import todo!
 - Build the data and stuff for the scout panel
 - Work out the system of scouts self fixing there own mistakes
 - Make the api calls and use that to make docummencation
Edit a whole matches data
Be able to sub auto
See teams pentlty avrage 
make snowgrave insert Errors into scoutwatch
Make snowgrave edit work
Delete Dups in database so we dont cause any issues so that we dont cause issues
be able to search by mvp
be able to find it avg
be able to find it by graph
assign pit scout
sumsititudte page
add todo

Edit:
    Edit the data
    //Cur pending is false marked is true along with redo
    set done in upcoming to true mark redo false
    preform check if not done then ret
    //now we know that check is ok check now gives us a list of scouters that are wrong
    for scouter in scouter wrong:
        mark done false and mark redo true
        //we dont need to mark dup, marked, and pending as they are already in a faled state
    for scouters in scouters_now_right:
        //mark done true and redo false no need this state is ok
        mark  marked false, dup (only once per type) false, and pending false
    ret 