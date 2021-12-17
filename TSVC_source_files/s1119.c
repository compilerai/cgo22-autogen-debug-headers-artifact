#include "globals.h"

int s1119()
{

//	linear dependence testing
//	no dependence - vectorizable
		for (int i = 1; i < LEN2; i++) {
			for (int j = 0; j < LEN2; j++) {
				aa[i][j] = aa[i-1][j] + bb[i][j];
			}
		}
	return 0;
}
