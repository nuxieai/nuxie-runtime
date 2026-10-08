#ifndef gd
#define gd e
#endif
#ifndef i7
#define i7 c
#endif
f float O8(c k,c b){float rg=dot(k,b);float hd=dot(k,k)*dot(b,b);return(hd==.0)?1.:clamp(rg*inversesqrt(hd),-1.,1.);}f void sg(c A0,c D0,c J0,c U0,c1(c) A,c1(c) J,c1(c) y2){y2=D0-A0;c j7=J0-D0;c P8=U0-A0;J=j7-y2;A=-3.*j7+P8;}f W Na(c A0,c D0,c J0,c U0){W t;t[0]=(any(notEqual(A0,D0))?D0:any(notEqual(D0,J0))?J0:U0)-A0;t[1]=U0-(any(notEqual(U0,J0))?J0:any(notEqual(J0,D0))?D0:A0);return t;}f float tg(c A0,c D0,c J0,c U0,float F1,float ug){c A,J,y2;sg(A0,D0,J0,U0,A,J,y2);c k7=3.*(((A*F1)+2.*J)*F1+y2);float id=length(k7);if(id==.0){return.0;}k7*=1./id;float Q8=2.*dot(A,k7);float l7=3.*(Q8*F1+4.*dot(J,k7))*F1+6.*dot(y2,k7);float Oa=min(F1,1.-F1);float vg=(Q8*Oa*Oa+l7)*Oa;float jd=min(ug,vg*.9999);float o3;if(Q8==.0){o3=jd/l7;}else{float R=1./Q8;float b=l7*R,S1=-jd*R;float m7=(-1./3.)*b,n7=.5*S1;float kd=n7*n7-m7*m7*m7;if(kd<.0){float R8=sqrt(m7);float h1=acos(n7/(R8*R8*R8));o3=-2.*R8*cos(h1*(1./3.)+(-n4*2./3.));}else{float A=pow(abs(n7)+sqrt(kd),1./3.);if(n7<.0) A=-A;o3=A!=.0?A+m7/A:.0;}}o3=abs(o3);e t0011=F1+gd(-o3,-o3,o3,o3);e ld=(A.xyxy*t0011+2.*J.xyxy)*t0011+y2.xyxy;W z2=Na(A0,D0,J0,U0);c wg=t0011.x<1e-3?z2[0]:ld.xy;c xg=t0011.z>1.-1e-3?z2[1]:ld.zw;return acos(O8(wg,xg));}f float S8(float k,float b){k=b<.0?-k:k;b=abs(b);return k>.0?(k<b?k/b:1.):.0;}float yg(c A0,c D0,c J0,c U0,c1(float) Pa){c md=U0-A0;float nd=length(U0-A0);if(nd==.0){Pa=.5;return.0;}c P1=i7(-md.y,md.x)/nd;float od=dot(P1,J0-A0);float U4=dot(P1,D0-A0);float V4=U4-od;
#if 0
float k=3.*V4;float pd=V4+U4;float S1=U4;float J2=sqrt(max(V4*V4+od*U4,.0));if(pd<.0) J2=-J2;J2+=pd;c o7=i7(S8(J2,k),S8(S1,J2));c n6=3.*(o7*(o7*(o7*V4-(U4+V4))+U4));n6=abs(n6);Pa=n6.x>n6.y?o7.x:o7.y;return max(n6.x,n6.y);
#else
float qd=3.*V4;float J=-U4-V4;float y2=U4;float t=.5;for(int M0=0;M0<3;++M0){float rd=qd*t;t=S8(rd*t-y2,2.*(rd+J));}Pa=t;return abs(t*(t*(t*qd+3.*J)+3.*y2));
#endif
}
