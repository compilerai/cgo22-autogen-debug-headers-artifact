#include "globals.h"

int s1115()
{

//	linear dependence testing
//	triangular saxpy loop
		for (int i = 0; i < LEN2; i++) {
			for (int j = 0; j < LEN2; j++) {
				aa[i][j] = aa[i][j]*cc[j][i] + bb[i][j];
			}
		}
	return 0;
}
