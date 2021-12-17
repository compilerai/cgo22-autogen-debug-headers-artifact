#include "globals.h"

int s132()
{
//	global data flow analysis
//	loop with multiple dimension ambiguous subscripts


	int m = 0;
	int j = m;
	int k = m+1;
		for (int i= 1; i < LEN2; i++) {
			aa[j][i] = aa[k][i-1] + b[i] * c[1];
		}
	return 0;
}
