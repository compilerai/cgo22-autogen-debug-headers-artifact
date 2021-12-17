#include "globals.h"

int s2244()
{

//	node splitting
//	cycle with ture and anti dependency

		for (int i = 0; i < LEN-1; i++) {
			a[i+1] = b[i] + e[i];
			a[i] = b[i] + c[i];
		}
	return 0;
}

