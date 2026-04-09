Some SQL notes:

Remember to use FOR NO KEY UPDATE in your select if you’re gonna write back the modified result (obviously in the same transaction)
To avoid deadlock, for now, when a single table is used, lock first the lowest id, then the higher one in order. What to do if lock across multiple table is needed is not yet defined.
