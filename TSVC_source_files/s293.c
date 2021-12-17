#include "globals.h"

int s293()
{

//	loop peeling
//	a(i)=a(0) with actual dependence cycle, loop is vectorizable

		for (int i = 0; i < LEN; i++) {
			a[i] = a[0];
		}
	return 0;
}
