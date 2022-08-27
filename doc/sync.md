
# Synchronisation between local sqlite & Checkvist server
## Algo
  
  Based roughly on https://unterwaditzer.net/2016/sync-algorithm

```
Operation spec:
  Task [id and updated contents if any]
  Source to apply op to (R[emote] | L[ocal] | S[yncstate])
  Action (Add | Delete | Update)
Get list of ops from Sync()  
  Upload R Ops
  Persist L Ops
  Persist S Ops
    (could be a separate list, or sync fields added to L persistence)
    

Sync(): 
  for one list (extend to list of lists later)
    Build list of ops, one each for R, L, S

    Starting point:
      R = get tasks from remote server
      L = get local tasks
      S = get syncstate tasks

    Make comparisons:
      RAdded = R, not in L or S 
        Add to L and  S
      RDeleted = L & S, not in R
        Delete from L  and S
      
      LAdded = L, not in R or S
        Add to R and S
      LDeleted = R and S, not in L
        Delete from R
      
      [mutation passes]
      LRSyncd[] = R and L (S or not) 
      (L updates, R updates) = HandlePotentialConflict(LRsyncd)


// should return list of ops (Tasks to update L/R)
HandlePotentialConflict (TR, TL)[]:
   For each TR/TL pair:
    (TResolved L|R) = if TR.tag = TL.tag, T
        else ResolveConflict(TR, TL)
    if T not in S, (TResolved, S) to collection
  Return collection of (TResolved/ L|R|S update tuples)

ResolveConflict(TR, TL):
  depends on policy
  Possible policies:
    - R always wins
    - etag <> comparison
```
  
### Cases

 * Task delete (one or both sides)

   Task id will be present in L/R &S but not in R/L
   Rdelete: Op: {id, L/S, Delete}
   Ldelete: Op: {id, R/S, Delete}
 
 * Task edit conflicts with task edit or delete on other side.

 * Task edit (one side)

 * Task add  (one side)

## Implementation

* reconciliation in mem (Rust) or db (SQL)?

  Would be a no-brainer for large data (ie. SQL), but we're unlikely to have that here. 

  Will try the latter first in any case. We want clients to operate primarily from the database, so it's inherently simple to sync via the db.

 * sqlite via Diesel or SQLx?
   
   Diesel tempting because it seems to be widely used and has good docs and isn't async. But I'll try SQLx's sql-centric approach first. There's enough to learn in Rust as it is without replacing SQL with an idiosyncratic ORM languageish.

 # Refs

 * https://unterwaditzer.net/2016/sync-algorithm
   closeish scenario as both involve a single canonical server
   
* https://hackernoon.com/operational-transformation-the-real-time-collaborative-editing-algorithm-bf8756683f66
  though probably not relevant - we don't really need to reconcile multiple clients editing the same items

* https://tech.trello.com/sync-architecture/
