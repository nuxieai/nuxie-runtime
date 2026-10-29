#ifndef td
#define td e
#endif
#ifndef m7
#define m7 c
#endif
f float X8(c l,c b){float Fg=dot(l,b);float ud=dot(l,l)*dot(b,b);return(ud==.0)?1.:clamp(Fg*inversesqrt(ud),-1.,1.);}f void Gg(c A0,c F0,c L0,c S0,k1(c) B,k1(c) J,k1(c) B2){B2=F0-A0;c n7=L0-F0;c Y8=S0-A0;J=n7-B2;B=-3.*n7+Y8;}f X Za(c A0,c F0,c L0,c S0){X t;t[0]=(any(notEqual(A0,F0))?F0:any(notEqual(F0,L0))?L0:S0)-A0;t[1]=S0-(any(notEqual(S0,L0))?L0:any(notEqual(L0,F0))?F0:A0);return t;}f float Hg(c A0,c F0,c L0,c S0,float J1,float Ig){c B,J,B2;Gg(A0,F0,L0,S0,B,J,B2);c o7=3.*(((B*J1)+2.*J)*J1+B2);float vd=length(o7);if(vd==.0){return.0;}o7*=1./vd;float Z8=2.*dot(B,o7);float p7=3.*(Z8*J1+4.*dot(J,o7))*J1+6.*dot(B2,o7);float ab=min(J1,1.-J1);float Jg=(Z8*ab*ab+p7)*ab;float wd=min(Ig,Jg*.9999);float q3;if(Z8==.0){q3=wd/p7;}else{float U=1./Z8;float b=p7*U,W1=-wd*U;float q7=(-1./3.)*b,r7=.5*W1;float xd=r7*r7-q7*q7*q7;if(xd<.0){float a9=sqrt(q7);float f1=acos(r7/(a9*a9*a9));q3=-2.*a9*cos(f1*(1./3.)+(-w4*2./3.));}else{float B=pow(abs(r7)+sqrt(xd),1./3.);if(r7<.0) B=-B;q3=B!=.0?B+q7/B:.0;}}q3=abs(q3);e t0011=J1+td(-q3,-q3,q3,q3);e yd=(B.xyxy*t0011+2.*J.xyxy)*t0011+B2.xyxy;X C2=Za(A0,F0,L0,S0);c Kg=t0011.x<1e-3?C2[0]:yd.xy;c Lg=t0011.z>1.-1e-3?C2[1]:yd.zw;return acos(X8(Kg,Lg));}f float c9(float l,float b){l=b<.0?-l:l;b=abs(b);return l>.0?(l<b?l/b:1.):.0;}float Mg(c A0,c F0,c L0,c S0,k1(float) bb){c zd=S0-A0;float Ad=length(S0-A0);if(Ad==.0){bb=.5;return.0;}c S1=m7(-zd.y,zd.x)/Ad;float Bd=dot(S1,L0-A0);float Z4=dot(S1,F0-A0);float a5=Z4-Bd;
#if 0
float l=3.*a5;float Cd=a5+Z4;float W1=Z4;float L2=sqrt(max(a5*a5+Bd*Z4,.0));if(Cd<.0) L2=-L2;L2+=Cd;c v7=m7(c9(L2,l),c9(W1,L2));c q6=3.*(v7*(v7*(v7*a5-(Z4+a5))+Z4));q6=abs(q6);bb=q6.x>q6.y?v7.x:v7.y;return max(q6.x,q6.y);
#else
float Dd=3.*a5;float J=-Z4-a5;float B2=Z4;float t=.5;for(int P0=0;P0<3;++P0){float Ed=Dd*t;t=c9(Ed*t-B2,2.*(Ed+J));}bb=t;return abs(t*(t*(t*Dd+3.*J)+3.*B2));
#endif
}
