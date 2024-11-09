%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Main_Container_Bool_Bool = type { %struct.Bool_Bool }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_Container_Bool_Bool(ptr noundef %item, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Container_Bool_Bool, align 4
   %field0 = getelementptr inbounds %struct.Main_Container_Bool_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %item, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %b0i7 = alloca %struct.siko_Tuple_, align 4
   %b_1 = alloca %struct.Bool_Bool, align 4
   %b0i5 = alloca %struct.Bool_Bool, align 4
   %b0i4 = alloca %struct.Main_Container_Bool_Bool, align 4
   %a_0 = alloca %struct.Main_Container_Bool_Bool, align 4
   %b0i2 = alloca %struct.Main_Container_Bool_Bool, align 4
   %b0i1 = alloca %struct.Bool_Bool, align 4
   call void @Bool_Bool_True(ptr %b0i1)
   call void @Main_Container_Bool_Bool(ptr %b0i1, ptr %b0i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_0, ptr align 4 %b0i2, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b0i4, ptr align 4 %a_0, i64 4, i1 false)
   call void @Main_other_Bool_Bool(ptr %b0i4, ptr %b0i5)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b_1, ptr align 4 %b0i5, i64 4, i1 false)
   call void @siko_Tuple_(ptr %b0i7)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i7, i64 0, i1 false)
   ret void
}

define private void @Main_other_Bool_Bool(ptr noundef %c, ptr noundef %fn_result) {
block0:
   %b0i1 = alloca %struct.Main_Container_Bool_Bool, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b0i1, ptr align 4 %c, i64 4, i1 false)
   %b0i2 = getelementptr inbounds %struct.Main_Container_Bool_Bool, ptr %b0i1, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i2, i64 4, i1 false)
   ret void
}

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_True, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


