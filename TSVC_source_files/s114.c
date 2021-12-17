#include "globals.h"

int s114()
{

//	linear dependence testing
//	transpose vectorization
//	Jump in data access - not vectorizable

		for (int i = 0; i < LEN2; i++) {
			for (int j = 0; j < i; j++) {
				aa[i][j] = aa[j][i] + bb[i][j];
			}
		}
	return 0;
}
