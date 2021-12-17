#include "globals.h"

int s119()
{

//	linear dependence testing
//	no dependence - vectorizable

		for (int i = 1; i < LEN2; i++) {
			for (int j = 1; j < LEN2; j++) {
				aa[i][j] = aa[i-1][j-1] + bb[i][j];
			}
		}
	return 0;
}
