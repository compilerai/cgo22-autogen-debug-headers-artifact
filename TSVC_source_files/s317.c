#include "globals.h"

int s317()
{

//	reductions
//	product reductio vectorize with
//	1. scalar expansion of factor, and product reduction
//	2. closed form solution: q = factor**n

	TYPE q;
		q = (TYPE)1.;
		for (int i = 0; i < LEN/2; i++) {
			q *= (TYPE)99;
		}
	temp = q;
	return 0;
}

